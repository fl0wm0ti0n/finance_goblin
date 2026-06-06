use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Utc};
use serde::Serialize;

use crate::forecast::repository::ForecastRepository;
use crate::forecast::types::fmt_amount;

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioHorizon {
    pub months: u32,
    pub value_eur: String,
    pub value_p10: Option<String>,
    pub value_p90: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioForecastResponse {
    pub horizons: Vec<PortfolioHorizon>,
    pub low_confidence: bool,
    pub fx_incomplete_warning: bool,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

pub struct PortfolioForecastService {
    forecast_repo: Arc<ForecastRepository>,
}

impl PortfolioForecastService {
    pub fn new(forecast_repo: Arc<ForecastRepository>) -> Self {
        Self { forecast_repo }
    }

    pub async fn latest(
        &self,
        fx_incomplete: bool,
    ) -> Result<PortfolioForecastResponse, sqlx::Error> {
        let ml = self
            .forecast_repo
            .latest_successful_by_kind("ml_enhanced")
            .await?;

        let Some(ml_row) = ml else {
            return Ok(PortfolioForecastResponse {
                horizons: vec![],
                low_confidence: false,
                fx_incomplete_warning: fx_incomplete,
                skipped: true,
                skip_reason: Some("no_ml_computation".into()),
            });
        };

        if ml_row
            .metadata
            .get("portfolio_forecast_skipped")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
        {
            return Ok(PortfolioForecastResponse {
                horizons: vec![],
                low_confidence: ml_row
                    .metadata
                    .get("low_confidence")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                fx_incomplete_warning: fx_incomplete,
                skipped: true,
                skip_reason: Some("insufficient_history".into()),
            });
        }

        let weekly = self
            .forecast_repo
            .fetch_portfolio_weekly(ml_row.id)
            .await?;

        if weekly.is_empty() {
            return Ok(PortfolioForecastResponse {
                horizons: vec![],
                low_confidence: false,
                fx_incomplete_warning: fx_incomplete,
                skipped: true,
                skip_reason: Some("no_portfolio_rows".into()),
            });
        }

        let today = Utc::now().date_naive();
        let horizons: Vec<PortfolioHorizon> = [3u32, 6, 12]
            .iter()
            .filter_map(|months| {
                horizon_value(&weekly, today, *months).map(|(v, p10, p90)| PortfolioHorizon {
                    months: *months,
                    value_eur: fmt_amount(v),
                    value_p10: p10.map(fmt_amount),
                    value_p90: p90.map(fmt_amount),
                })
            })
            .collect();

        Ok(PortfolioForecastResponse {
            low_confidence: ml_row
                .metadata
                .get("low_confidence")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            fx_incomplete_warning: fx_incomplete,
            skipped: horizons.is_empty(),
            skip_reason: if horizons.is_empty() {
                Some("no_horizon_match".into())
            } else {
                None
            },
            horizons,
        })
    }
}

fn horizon_value(
    weekly: &[crate::forecast::types::PortfolioWeeklyPoint],
    today: NaiveDate,
    months: u32,
) -> Option<(f64, Option<f64>, Option<f64>)> {
    let target = add_months(today, months as i64);
    weekly
        .iter()
        .min_by_key(|p| (p.ts - target).num_days().unsigned_abs())
        .map(|p| (p.value_eur, p.value_p10, p.value_p90))
}

fn add_months(date: NaiveDate, months: i64) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months as i32;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) + 1;
    NaiveDate::from_ymd_opt(y, m as u32, 1).unwrap()
}
