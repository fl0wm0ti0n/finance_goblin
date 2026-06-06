use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::{AlertsConfig, ForecastMlConfig};
use crate::forecast::repository::ForecastRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    pub balance_stress: f64,
    pub plan_viability: f64,
    pub crypto_volatility: f64,
    pub ml_divergence_modifier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRiskScore {
    pub plan_computation_id: Uuid,
    pub score: i16,
    pub band: String,
    pub components: RiskComponents,
}

#[derive(Clone)]
pub struct PlanRiskService {
    pool: PgPool,
    forecast_repo: Arc<ForecastRepository>,
    alerts_config: AlertsConfig,
    forecast_ml_config: ForecastMlConfig,
}

impl PlanRiskService {
    pub fn new(
        pool: PgPool,
        forecast_repo: Arc<ForecastRepository>,
        alerts_config: AlertsConfig,
        forecast_ml_config: ForecastMlConfig,
    ) -> Self {
        Self {
            pool,
            forecast_repo,
            alerts_config,
            forecast_ml_config,
        }
    }

    pub async fn compute(&self, plan_computation_id: Uuid) -> Result<PlanRiskScore, sqlx::Error> {
        let active = self.load_active_plan_context(plan_computation_id).await?;
        let balance_stress = self.compute_balance_stress(&active).await?;
        let plan_viability = self.compute_plan_viability(&active).await?;
        let crypto_volatility = self.compute_crypto_volatility().await?;
        let ml_divergence = self.compute_ml_divergence_modifier().await?;

        let raw = 0.45 * balance_stress
            + 0.40 * plan_viability
            + 0.15 * crypto_volatility
            + ml_divergence;
        let score = raw.round().clamp(0.0, 100.0) as i16;
        let band = band_for_score(score);

        let components = RiskComponents {
            balance_stress,
            plan_viability,
            crypto_volatility,
            ml_divergence_modifier: ml_divergence,
        };

        sqlx::query(
            r#"
            INSERT INTO plan_risk_scores (plan_computation_id, score, band, components)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (plan_computation_id) DO UPDATE SET
                score = EXCLUDED.score,
                band = EXCLUDED.band,
                components = EXCLUDED.components,
                computed_at = NOW()
            "#,
        )
        .bind(plan_computation_id)
        .bind(score)
        .bind(&band)
        .bind(json!(components))
        .execute(&self.pool)
        .await?;

        Ok(PlanRiskScore {
            plan_computation_id,
            score,
            band,
            components,
        })
    }

    pub async fn latest_for_active_plan(&self) -> Result<Option<PlanRiskScore>, sqlx::Error> {
        let row = sqlx::query_as::<_, RiskRow>(
            r#"
            SELECT prs.plan_computation_id, prs.score, prs.band, prs.components
            FROM plan_risk_scores prs
            JOIN plan_computations pc ON pc.id = prs.plan_computation_id
            JOIN plan_versions v ON v.id = pc.version_id
            JOIN plans p ON p.id = v.plan_id
            WHERE p.is_active = true AND v.is_latest = true AND pc.status = 'success'
            ORDER BY prs.computed_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| PlanRiskScore {
            plan_computation_id: r.plan_computation_id,
            score: r.score,
            band: r.band,
            components: serde_json::from_value(r.components).unwrap_or(RiskComponents {
                balance_stress: 0.0,
                plan_viability: 0.0,
                crypto_volatility: 0.0,
                ml_divergence_modifier: 0.0,
            }),
        }))
    }

    async fn load_active_plan_context(
        &self,
        plan_computation_id: Uuid,
    ) -> Result<ActivePlanContext, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Uuid)>(
            r#"
            SELECT v.id, v.plan_id
            FROM plan_computations pc
            JOIN plan_versions v ON v.id = pc.version_id
            WHERE pc.id = $1
            "#,
        )
        .bind(plan_computation_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ActivePlanContext {
            version_id: row.0,
            plan_id: row.1,
            plan_computation_id,
        })
    }

    async fn compute_balance_stress(&self, ctx: &ActivePlanContext) -> Result<f64, sqlx::Error> {
        let today = Utc::now().date_naive();
        let end = add_months(today, 6);
        let rows: Vec<(NaiveDate, f64)> = sqlx::query_as(
            r#"
            SELECT ts::date AS day, planned_balance::float8 AS bal
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
              AND ts::date >= $3 AND ts::date <= $4
              AND EXTRACT(day FROM ts + INTERVAL '1 day') = 1
            ORDER BY day
            "#,
        )
        .bind(ctx.version_id)
        .bind(ctx.plan_computation_id)
        .bind(today)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        let negatives = rows.iter().filter(|(_, b)| *b < 0.0).count();
        Ok((negatives as f64 / 6.0).min(1.0) * 100.0)
    }

    async fn compute_plan_viability(&self, ctx: &ActivePlanContext) -> Result<f64, sqlx::Error> {
        let today = Utc::now().date_naive();
        let current_month_end = month_end(today);
        let next_month_end = month_end(add_months(today, 1));

        let rows: Vec<(NaiveDate, f64)> = sqlx::query_as(
            r#"
            SELECT ts::date AS day, planned_balance::float8 AS bal
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
              AND ts::date IN ($3, $4)
            "#,
        )
        .bind(ctx.version_id)
        .bind(ctx.plan_computation_id)
        .bind(current_month_end)
        .bind(next_month_end)
        .fetch_all(&self.pool)
        .await?;

        let current = rows
            .iter()
            .find(|(d, _)| *d == current_month_end)
            .map(|(_, b)| *b);
        let next = rows
            .iter()
            .find(|(d, _)| *d == next_month_end)
            .map(|(_, b)| *b);

        let primary = current.map(|b| b < 0.0).unwrap_or(false);
        let consecutive =
            primary && next.map(|b| b < 0.0).unwrap_or(false);

        if consecutive {
            Ok(100.0)
        } else if primary {
            Ok(80.0)
        } else {
            Ok(0.0)
        }
    }

    async fn compute_crypto_volatility(&self) -> Result<f64, sqlx::Error> {
        let rows: Vec<f64> = sqlx::query_scalar(
            r#"
            SELECT crypto_value_eur::float8
            FROM portfolio_pnl_snapshots
            ORDER BY snapshot_date DESC
            LIMIT 12
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 2 {
            return Ok(0.0);
        }

        let mean = rows.iter().sum::<f64>() / rows.len() as f64;
        if mean.abs() < 1e-9 {
            return Ok(0.0);
        }
        let variance = rows
            .iter()
            .map(|v| {
                let pct = (v - mean) / mean;
                pct * pct
            })
            .sum::<f64>()
            / rows.len() as f64;
        let cv = variance.sqrt();
        Ok((cv * 100.0).min(100.0))
    }

    async fn compute_ml_divergence_modifier(&self) -> Result<f64, sqlx::Error> {
        if !self.forecast_ml_config.enabled {
            return Ok(0.0);
        }

        let baseline = self.forecast_repo.latest_successful_by_kind("baseline").await?;
        let ml = self.forecast_repo.latest_successful_by_kind("ml_enhanced").await?;
        let (Some(base_row), Some(ml_row)) = (baseline, ml) else {
            return Ok(0.0);
        };

        let today = Utc::now().date_naive();
        let end = month_end(add_months(today, 6));

        let base_series = self
            .forecast_repo
            .fetch_daily_series(base_row.id, "household", Some(today), Some(end))
            .await?;
        let ml_series = self
            .forecast_repo
            .fetch_daily_series_with_bands(ml_row.id, "household", Some(today), Some(end))
            .await?;

        let base_end = base_series.last().map(|p| p.balance).unwrap_or(0.0);
        let ml_p10 = ml_series
            .last()
            .and_then(|p| p.balance_p10)
            .unwrap_or(base_end);

        let threshold = self.alerts_config.scarcity_threshold_eur;
        if ml_p10 < threshold && base_end >= threshold {
            Ok(5.0)
        } else if ml_p10 >= threshold && base_end < threshold {
            Ok(-5.0)
        } else {
            Ok(0.0)
        }
    }
}

struct ActivePlanContext {
    version_id: Uuid,
    plan_id: Uuid,
    plan_computation_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct RiskRow {
    plan_computation_id: Uuid,
    score: i16,
    band: String,
    components: Value,
}

fn band_for_score(score: i16) -> String {
    if score <= 29 {
        "low".into()
    } else if score <= 59 {
        "medium".into()
    } else {
        "high".into()
    }
}

fn month_end(date: NaiveDate) -> NaiveDate {
    let (y, m) = (date.year(), date.month());
    if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap() - chrono::Duration::days(1)
    }
}

fn add_months(date: NaiveDate, months: i64) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months as i32;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) + 1;
    let day = date.day().min(28);
    NaiveDate::from_ymd_opt(y, m as u32, day).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_thresholds() {
        assert_eq!(band_for_score(10), "low");
        assert_eq!(band_for_score(40), "medium");
        assert_eq!(band_for_score(80), "high");
    }

    #[test]
    fn raw_score_clamp() {
        let raw = 0.45_f64 * 100.0 + 0.40 * 100.0 + 0.15 * 100.0 + 5.0;
        let score = raw.round().clamp(0.0, 100.0) as i16;
        assert_eq!(score, 100);
    }
}
