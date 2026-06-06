use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::{json, Value};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::ForecastMlConfig;
use crate::forecast::repository::ForecastRepository;
use crate::forecast::types::{DailyPointWithBands, PortfolioWeeklyPoint};

use super::overlay::overlay_monthly_onto_baseline;
use super::sidecar::{ForecastRequest, SidecarClient, SidecarError, SidecarPoint};

#[derive(Clone)]
pub struct ForecastMlService {
    repo: Arc<ForecastRepository>,
    config: ForecastMlConfig,
    sidecar: SidecarClient,
}

#[derive(Debug, thiserror::Error)]
pub enum ForecastMlError {
    #[error("ml disabled")]
    Disabled,
    #[error("sidecar unavailable")]
    SidecarUnavailable,
    #[error("insufficient history")]
    InsufficientHistory,
    #[error("sidecar error: {0}")]
    Sidecar(String),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}

impl ForecastMlService {
    pub fn new(repo: Arc<ForecastRepository>, config: ForecastMlConfig) -> Self {
        let sidecar = SidecarClient::new(&config);
        Self {
            repo,
            config,
            sidecar,
        }
    }

    pub fn config(&self) -> &ForecastMlConfig {
        &self.config
    }

    pub async fn recompute(
        &self,
        sync_run_id: Uuid,
        baseline_id: Uuid,
    ) -> Result<Option<Uuid>, ForecastMlError> {
        if !self.config.enabled {
            return Err(ForecastMlError::Disabled);
        }
        if !self.sidecar.health_ok().await {
            return Err(ForecastMlError::SidecarUnavailable);
        }

        let accounts = self.repo.list_asset_accounts().await?;
        let mut all_monthly_points = 0usize;
        for account in &accounts {
            let hist = self
                .repo
                .fetch_historical_monthly_net_cashflow(&account.firefly_id, 36)
                .await?;
            all_monthly_points = all_monthly_points.max(hist.len());
        }

        if (all_monthly_points as u32) < self.config.min_monthly_points {
            return Err(ForecastMlError::InsufficientHistory);
        }

        let ml_id = Uuid::new_v4();
        self.repo
            .insert_ml_computation(ml_id, sync_run_id, baseline_id)
            .await?;

        let started = std::time::Instant::now();
        let mut metadata = json!({
            "ml_status": "success",
            "ml_skipped_reason": null,
            "portfolio_forecast_skipped": true,
        });

        let mut model_family = String::new();
        let mut seasonal_periods: Vec<u32> = vec![];
        let mut seasonal_strength = 0.0;
        let mut seasonal_detected = false;
        let mut backtest_wmape: Option<f64> = None;
        let mut low_confidence = false;

        for account in &accounts {
            let account_id = &account.firefly_id;
            let baseline_daily = self
                .repo
                .fetch_daily_series(baseline_id, account_id, None, None)
                .await?;
            let baseline_monthly = self
                .repo
                .fetch_monthly_series(baseline_id, account_id)
                .await?;
            let hist = self
                .repo
                .fetch_historical_monthly_net_cashflow(account_id, 36)
                .await?;

            let points: Vec<SidecarPoint> = hist
                .iter()
                .map(|(d, y)| SidecarPoint {
                    ds: d.format("%Y-%m-%d").to_string(),
                    y: *y,
                })
                .collect();

            let sidecar_resp = self
                .sidecar
                .forecast(&ForecastRequest {
                    series_id: account_id.clone(),
                    freq: "MS".into(),
                    points,
                    horizon: 24,
                    level: vec![self.config.interval_level],
                    model: "auto".into(),
                })
                .await
                .map_err(|e| match e {
                    SidecarError::InsufficientHistory => ForecastMlError::InsufficientHistory,
                    SidecarError::Unavailable | SidecarError::Timeout => {
                        ForecastMlError::SidecarUnavailable
                    }
                    SidecarError::Http(msg) => ForecastMlError::Sidecar(msg),
                })?;

            model_family = sidecar_resp.model_family.clone();
            seasonal_periods = sidecar_resp.seasonal_periods.clone();
            seasonal_strength = sidecar_resp.seasonal_strength;
            seasonal_detected = sidecar_resp.seasonal_detected;
            backtest_wmape = sidecar_resp.backtest_wmape;
            low_confidence = sidecar_resp.low_confidence;

            let overlay = overlay_monthly_onto_baseline(
                &baseline_daily,
                &baseline_monthly,
                &sidecar_resp.forecasts,
            );
            self.repo
                .bulk_insert_daily_with_bands(ml_id, account_id, &overlay)
                .await?;
        }

        // Household aggregate series
        self.persist_household_overlay(ml_id, baseline_id).await?;

        metadata["model_family"] = json!(model_family);
        metadata["seasonal_periods"] = json!(seasonal_periods);
        metadata["seasonal_detected"] = json!(seasonal_detected);
        metadata["seasonal_strength"] = json!(seasonal_strength);
        metadata["backtest_wmape"] = json!(backtest_wmape);
        metadata["low_confidence"] = json!(low_confidence);

        if let Err(e) = self.run_portfolio_forecast(ml_id, &mut metadata).await {
            warn!(?e, "portfolio forecast skipped");
            metadata["portfolio_forecast_skipped"] = json!(true);
        } else {
            metadata["portfolio_forecast_skipped"] = json!(false);
        }

        self.repo.mark_success(ml_id, &metadata).await?;
        self.repo.enforce_retention().await?;

        info!(
            %ml_id,
            %baseline_id,
            duration_ms = started.elapsed().as_millis(),
            "forecast_ml recompute succeeded"
        );
        Ok(Some(ml_id))
    }

    async fn persist_household_overlay(
        &self,
        ml_id: Uuid,
        baseline_id: Uuid,
    ) -> Result<(), ForecastMlError> {
        let accounts = self.repo.list_asset_accounts().await?;
        let mut by_date: std::collections::BTreeMap<NaiveDate, (f64, Option<f64>, Option<f64>)> =
            std::collections::BTreeMap::new();

        for account in accounts {
            let series = self
                .repo
                .fetch_daily_series_with_bands(ml_id, &account.firefly_id, None, None)
                .await?;
            for p in series {
                let entry = by_date.entry(p.date).or_insert((0.0, Some(0.0), Some(0.0)));
                entry.0 += p.balance;
                if let (Some(lo), Some(hi)) = (p.balance_p10, p.balance_p90) {
                    entry.1 = Some(entry.1.unwrap_or(0.0) + lo);
                    entry.2 = Some(entry.2.unwrap_or(0.0) + hi);
                }
            }
        }

        let household: Vec<DailyPointWithBands> = by_date
            .into_iter()
            .map(|(date, (balance, p10, p90))| DailyPointWithBands {
                date,
                balance,
                balance_p10: p10,
                balance_p90: p90,
            })
            .collect();

        if !household.is_empty() {
            self.repo
                .bulk_insert_daily_with_bands(ml_id, "household", &household)
                .await?;
        }

        let _ = baseline_id;
        Ok(())
    }

    async fn run_portfolio_forecast(
        &self,
        ml_id: Uuid,
        metadata: &mut Value,
    ) -> Result<(), ForecastMlError> {
        let hist = self.repo.fetch_historical_crypto_weekly(52).await?;
        if (hist.len() as u32) < self.config.min_portfolio_weeks {
            return Err(ForecastMlError::InsufficientHistory);
        }

        let points: Vec<SidecarPoint> = hist
            .iter()
            .map(|(d, y)| SidecarPoint {
                ds: d.format("%Y-%m-%d").to_string(),
                y: *y,
            })
            .collect();

        let resp = self
            .sidecar
            .forecast(&ForecastRequest {
                series_id: "crypto_portfolio".into(),
                freq: "W".into(),
                points,
                horizon: 52,
                level: vec![self.config.interval_level],
                model: "auto".into(),
            })
            .await
            .map_err(|e| match e {
                SidecarError::InsufficientHistory => ForecastMlError::InsufficientHistory,
                SidecarError::Unavailable | SidecarError::Timeout => {
                    ForecastMlError::SidecarUnavailable
                }
                SidecarError::Http(msg) => ForecastMlError::Sidecar(msg),
            })?;

        let weekly: Vec<PortfolioWeeklyPoint> = resp
            .forecasts
            .iter()
            .filter_map(|fc| {
                NaiveDate::parse_from_str(&fc.ds, "%Y-%m-%d")
                    .ok()
                    .map(|ts| PortfolioWeeklyPoint {
                        ts,
                        value_eur: fc.y,
                        value_p10: Some(fc.y_lo),
                        value_p90: Some(fc.y_hi),
                    })
            })
            .collect();

        self.repo.bulk_insert_portfolio_weekly(ml_id, &weekly).await?;
        Ok(())
    }

    pub async fn record_skip_on_baseline(
        &self,
        baseline_id: Uuid,
        err: &ForecastMlError,
    ) -> Result<(), sqlx::Error> {
        let reason = match err {
            ForecastMlError::Disabled => "sidecar_disabled",
            ForecastMlError::SidecarUnavailable => "sidecar_unavailable",
            ForecastMlError::InsufficientHistory => "insufficient_history",
            ForecastMlError::Sidecar(msg) => {
                return self.record_skip_reason(baseline_id, "sidecar_error", Some(msg)).await;
            }
            ForecastMlError::Db(_) => "sidecar_error",
        };
        self.record_skip_reason(baseline_id, reason, None).await
    }

    pub async fn record_skip_on_error(
        &self,
        baseline_id: Uuid,
        err: &str,
    ) -> Result<(), sqlx::Error> {
        self.record_skip_reason(baseline_id, "sidecar_error", Some(err))
            .await
    }

    async fn record_skip_reason(
        &self,
        baseline_id: Uuid,
        reason: &str,
        detail: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let patch = json!({
            "ml_status": "skipped",
            "ml_skipped_reason": reason,
            "ml_skip_detail": detail,
        });
        self.repo.merge_metadata(baseline_id, &patch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ForecastMlConfig;

    #[test]
    fn skip_reason_disabled_by_default() {
        assert!(!ForecastMlConfig::default().enabled);
    }
}
