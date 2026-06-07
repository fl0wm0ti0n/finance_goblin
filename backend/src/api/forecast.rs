use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::forecast::repository::ForecastRepository;
use crate::forecast::types::fmt_amount;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct BalanceWarning {
    pub account_id: String,
    pub starting_balance: f64,
    pub reason: String,
}

#[derive(Serialize)]
pub struct MetaResponse {
    pub computation_id: Option<String>,
    pub computed_at: Option<String>,
    pub stale: bool,
    pub low_confidence: bool,
    pub sync_run_id: Option<String>,
    pub baseline_computation_id: Option<String>,
    pub ml_computation_id: Option<String>,
    pub ml_status: Option<String>,
    pub ml_skipped_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_warnings: Option<Vec<BalanceWarning>>,
    pub seasonal_detected: Option<bool>,
    pub seasonal_periods: Option<Vec<u32>>,
    pub seasonal_strength: Option<f64>,
    pub model_family: Option<String>,
    pub backtest_wmape: Option<f64>,
}

#[derive(Serialize)]
pub struct AccountItem {
    pub id: String,
    pub name: String,
    pub currency: Option<String>,
}

#[derive(Serialize)]
pub struct DailyResponse {
    pub milestones: DailyMilestones,
    pub series: Vec<DailyPointResponse>,
}

#[derive(Serialize)]
pub struct DailyMilestones {
    pub tomorrow: String,
    pub next_week: String,
    pub month_end: String,
}

#[derive(Serialize)]
pub struct DailyPointResponse {
    pub date: String,
    pub balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_p10: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_p90: Option<String>,
}

#[derive(Serialize)]
pub struct MonthlyResponse {
    pub series: Vec<MonthlyPointResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonal: Option<SeasonalMeta>,
}

#[derive(Serialize)]
pub struct SeasonalMeta {
    pub seasonal_detected: bool,
    pub seasonal_periods: Vec<u32>,
    pub seasonal_strength: Option<f64>,
}

#[derive(Serialize)]
pub struct BucketSources {
    pub income: String,
    pub fixed_costs: String,
    pub variable_costs: String,
}

#[derive(Serialize)]
pub struct MonthlyPointResponse {
    pub month: String,
    pub income: String,
    pub fixed_costs: String,
    pub variable_costs: String,
    pub free_cashflow: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_sources: Option<BucketSources>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ai_mapped: bool,
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Serialize)]
pub struct LongTermResponse {
    pub variant: String,
    pub series: Vec<DailyPointResponse>,
    pub end_balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_balance_p10: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_balance_p90: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonal_periods: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtest_wmape: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_confidence: Option<bool>,
}

#[derive(Serialize)]
pub struct CompareResponse {
    pub horizon_months: i64,
    pub baseline: CompareSeries,
    pub ml_enhanced: Option<CompareSeries>,
    pub delta_end_balance: Option<String>,
    pub ml_available: bool,
    pub ml_skipped_reason: Option<String>,
}

#[derive(Serialize)]
pub struct CompareSeries {
    pub end_balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_balance_p10: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_balance_p90: Option<String>,
    pub series: Vec<DailyPointResponse>,
}

#[derive(Serialize)]
pub struct AggregateResponse {
    pub series: Vec<DailyPointResponse>,
}

#[derive(Deserialize)]
pub struct AccountQuery {
    pub account_id: String,
}

#[derive(Deserialize)]
pub struct LongTermQuery {
    pub account_id: String,
    pub horizon: Option<i64>,
    pub variant: Option<String>,
}

#[derive(Deserialize)]
pub struct CompareQuery {
    pub account_id: String,
    pub horizon: Option<i64>,
}

#[derive(Deserialize)]
pub struct AggregateQuery {
    pub horizon: Option<i64>,
}

pub async fn meta(State(state): State<Arc<AppState>>) -> Result<Json<MetaResponse>, StatusCode> {
    let repo = state.forecast.repository();
    let baseline = repo
        .latest_successful_by_kind("baseline")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ml = repo
        .latest_successful_by_kind("ml_enhanced")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let latest_any = repo
        .latest_any()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stale = repo.is_stale(&baseline, &latest_any);

    let Some(row) = baseline else {
        return Ok(Json(MetaResponse {
            computation_id: None,
            computed_at: None,
            stale: false,
            low_confidence: false,
            sync_run_id: None,
            baseline_computation_id: None,
            ml_computation_id: None,
            ml_status: None,
            ml_skipped_reason: None,
            balance_warnings: None,
            seasonal_detected: None,
            seasonal_periods: None,
            seasonal_strength: None,
            model_family: None,
            backtest_wmape: None,
        }));
    };

    let ml_meta = ml.as_ref().map(|m| &m.metadata);
    let baseline_meta = &row.metadata;

    let ml_status = ml_meta
        .and_then(|m| m.get("ml_status"))
        .or_else(|| baseline_meta.get("ml_status"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| ml.as_ref().map(|_| "success".into()));

    let ml_skipped = derive_ml_skipped_reason(
        baseline_meta,
        state.config.forecast_ml.enabled,
    );

    let balance_warnings = baseline_meta
        .get("balance_warnings")
        .and_then(|v| serde_json::from_value::<Vec<BalanceWarning>>(v.clone()).ok());

    Ok(Json(MetaResponse {
        computation_id: Some(row.id.to_string()),
        computed_at: Some(row.computed_at.to_rfc3339()),
        stale,
        low_confidence: ForecastRepository::metadata_low_confidence(&row.metadata),
        sync_run_id: row.sync_run_id.map(|id| id.to_string()),
        baseline_computation_id: Some(row.id.to_string()),
        ml_computation_id: ml.as_ref().map(|m| m.id.to_string()),
        ml_status,
        ml_skipped_reason: ml_skipped,
        balance_warnings,
        seasonal_detected: ml_meta
            .and_then(|m| m.get("seasonal_detected"))
            .and_then(|v| v.as_bool()),
        seasonal_periods: ml_meta.and_then(|m| {
            m.get("seasonal_periods")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
        }),
        seasonal_strength: ml_meta
            .and_then(|m| m.get("seasonal_strength"))
            .and_then(|v| v.as_f64()),
        model_family: ml_meta
            .and_then(|m| m.get("model_family"))
            .and_then(|v| v.as_str())
            .map(String::from),
        backtest_wmape: ml_meta
            .and_then(|m| m.get("backtest_wmape"))
            .and_then(|v| v.as_f64()),
    }))
}

pub async fn accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AccountItem>>, StatusCode> {
    let rows = state
        .forecast
        .repository()
        .list_asset_accounts()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|a| AccountItem {
                id: a.firefly_id,
                name: a.name.unwrap_or_else(|| "Unknown".into()),
                currency: a.currency,
            })
            .collect(),
    ))
}

pub async fn daily(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AccountQuery>,
) -> Result<Json<DailyResponse>, StatusCode> {
    let repo = state.forecast.repository();
    let computation = latest_baseline_computation(&repo).await?;
    let today = Utc::now().date_naive();
    let month_end = last_day_of_month(today);
    let series = repo
        .fetch_daily_series(computation, &q.account_id, Some(today), Some(month_end))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let full_series = repo
        .fetch_daily_series(computation, &q.account_id, None, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tomorrow = balance_on(&full_series, today + chrono::Duration::days(1));
    let next_week = balance_on(&full_series, today + chrono::Duration::days(7));
    let month_end_balance = balance_on(&full_series, month_end);

    Ok(Json(DailyResponse {
        milestones: DailyMilestones {
            tomorrow: fmt_amount(tomorrow),
            next_week: fmt_amount(next_week),
            month_end: fmt_amount(month_end_balance),
        },
        series: series
            .into_iter()
            .map(|p| DailyPointResponse {
                date: p.date.format("%Y-%m-%d").to_string(),
                balance: fmt_amount(p.balance),
                balance_p10: None,
                balance_p90: None,
            })
            .collect(),
    }))
}

pub async fn monthly(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AccountQuery>,
) -> Result<Json<MonthlyResponse>, StatusCode> {
    let repo = state.forecast.repository();
    let computation = latest_baseline_computation(&repo).await?;
    let series = repo
        .fetch_monthly_series(computation, &q.account_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ml = repo.latest_successful_by_kind("ml_enhanced").await.ok().flatten();
    let seasonal = ml.map(|m| {
        let meta = &m.metadata;
        SeasonalMeta {
            seasonal_detected: meta
                .get("seasonal_detected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            seasonal_periods: meta
                .get("seasonal_periods")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            seasonal_strength: meta.get("seasonal_strength").and_then(|v| v.as_f64()),
        }
    });

    Ok(Json(MonthlyResponse {
        series: series
            .into_iter()
            .map(|m| MonthlyPointResponse {
                month: m.month.format("%Y-%m-%d").to_string(),
                income: fmt_amount(m.income),
                fixed_costs: fmt_amount(m.fixed_costs),
                variable_costs: fmt_amount(m.variable_costs),
                free_cashflow: fmt_amount(m.free_cashflow),
                bucket_sources: m.bucket_sources.map(|s| BucketSources {
                    income: s.income,
                    fixed_costs: s.fixed_costs,
                    variable_costs: s.variable_costs,
                }),
                ai_mapped: m.ai_mapped,
            })
            .collect(),
        seasonal,
    }))
}

pub async fn long_term(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LongTermQuery>,
) -> Result<Json<LongTermResponse>, StatusCode> {
    let horizon = q.horizon.unwrap_or(12);
    if ![3, 6, 12, 24].contains(&horizon) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let variant = q.variant.as_deref().unwrap_or("baseline");
    if variant != "baseline" && variant != "ml_enhanced" {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state.forecast.repository();
    let today = Utc::now().date_naive();
    let end = add_months(today, horizon);
    let end_date = last_day_of_month(end);

    if variant == "ml_enhanced" {
        let ml = repo
            .latest_successful_by_kind("ml_enhanced")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let series = repo
            .fetch_daily_series_with_bands(ml.id, &q.account_id, Some(today), Some(end_date))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let last = series.last();
        let meta = &ml.metadata;

        return Ok(Json(LongTermResponse {
            variant: variant.into(),
            end_balance: fmt_amount(last.map(|p| p.balance).unwrap_or(0.0)),
            end_balance_p10: last.and_then(|p| p.balance_p10.map(fmt_amount)),
            end_balance_p90: last.and_then(|p| p.balance_p90.map(fmt_amount)),
            model_family: meta
                .get("model_family")
                .and_then(|v| v.as_str())
                .map(String::from),
            seasonal_periods: meta
                .get("seasonal_periods")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            backtest_wmape: meta.get("backtest_wmape").and_then(|v| v.as_f64()),
            low_confidence: meta.get("low_confidence").and_then(|v| v.as_bool()),
            series: series
                .into_iter()
                .map(|p| DailyPointResponse {
                    date: p.date.format("%Y-%m-%d").to_string(),
                    balance: fmt_amount(p.balance),
                    balance_p10: p.balance_p10.map(fmt_amount),
                    balance_p90: p.balance_p90.map(fmt_amount),
                })
                .collect(),
        }));
    }

    let computation = latest_baseline_computation(&repo).await?;
    let series = repo
        .fetch_daily_series(computation, &q.account_id, Some(today), Some(end_date))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let end_balance = series.last().map(|p| p.balance).unwrap_or(0.0);

    Ok(Json(LongTermResponse {
        variant: "baseline".into(),
        series: series
            .into_iter()
            .map(|p| DailyPointResponse {
                date: p.date.format("%Y-%m-%d").to_string(),
                balance: fmt_amount(p.balance),
                balance_p10: None,
                balance_p90: None,
            })
            .collect(),
        end_balance: fmt_amount(end_balance),
        end_balance_p10: None,
        end_balance_p90: None,
        model_family: None,
        seasonal_periods: None,
        backtest_wmape: None,
        low_confidence: None,
    }))
}

pub async fn compare(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CompareQuery>,
) -> Result<Json<CompareResponse>, StatusCode> {
    let horizon = q.horizon.unwrap_or(12);
    if ![6, 12, 24].contains(&horizon) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo = state.forecast.repository();
    let baseline_id = latest_baseline_computation(&repo).await?;
    let today = Utc::now().date_naive();
    let end_date = last_day_of_month(add_months(today, horizon));

    let base_series = repo
        .fetch_daily_series(baseline_id, &q.account_id, Some(today), Some(end_date))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let base_end = base_series.last().map(|p| p.balance).unwrap_or(0.0);

    let baseline = CompareSeries {
        end_balance: fmt_amount(base_end),
        end_balance_p10: None,
        end_balance_p90: None,
        series: base_series
            .iter()
            .map(|p| DailyPointResponse {
                date: p.date.format("%Y-%m-%d").to_string(),
                balance: fmt_amount(p.balance),
                balance_p10: None,
                balance_p90: None,
            })
            .collect(),
    };

    let baseline_row = repo
        .latest_successful_by_kind("baseline")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ml_skipped = baseline_row.as_ref().map(|r| {
        derive_ml_skipped_reason(&r.metadata, state.config.forecast_ml.enabled)
    }).flatten();

    let ml_row = repo
        .latest_successful_by_kind("ml_enhanced")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(ml) = ml_row else {
        return Ok(Json(CompareResponse {
            horizon_months: horizon,
            baseline,
            ml_enhanced: None,
            delta_end_balance: None,
            ml_available: false,
            ml_skipped_reason: ml_skipped.or(Some("no_ml_computation".into())),
        }));
    };

    let ml_series = repo
        .fetch_daily_series_with_bands(ml.id, &q.account_id, Some(today), Some(end_date))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ml_end = ml_series.last().map(|p| p.balance).unwrap_or(0.0);
    let delta = ml_end - base_end;

    Ok(Json(CompareResponse {
        horizon_months: horizon,
        baseline,
        ml_enhanced: Some(CompareSeries {
            end_balance: fmt_amount(ml_end),
            end_balance_p10: ml_series
                .last()
                .and_then(|p| p.balance_p10.map(fmt_amount)),
            end_balance_p90: ml_series
                .last()
                .and_then(|p| p.balance_p90.map(fmt_amount)),
            series: ml_series
                .iter()
                .map(|p| DailyPointResponse {
                    date: p.date.format("%Y-%m-%d").to_string(),
                    balance: fmt_amount(p.balance),
                    balance_p10: p.balance_p10.map(fmt_amount),
                    balance_p90: p.balance_p90.map(fmt_amount),
                })
                .collect(),
        }),
        delta_end_balance: Some(fmt_amount(delta)),
        ml_available: true,
        ml_skipped_reason: None,
    }))
}

pub async fn aggregate(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AggregateQuery>,
) -> Result<Json<AggregateResponse>, StatusCode> {
    let repo = state.forecast.repository();
    let computation = latest_baseline_computation(&repo).await?;
    let today = Utc::now().date_naive();
    let horizon = q.horizon.unwrap_or(12);
    let end = if [3, 6, 12, 24].contains(&horizon) {
        last_day_of_month(add_months(today, horizon))
    } else {
        today + chrono::Duration::days(730)
    };

    let mut points = state
        .forecast
        .aggregate_daily_balances(computation, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    points.retain(|p| p.date >= today && p.date <= end);

    Ok(Json(AggregateResponse {
        series: points
            .into_iter()
            .map(|p| DailyPointResponse {
                date: p.date.format("%Y-%m-%d").to_string(),
                balance: fmt_amount(p.balance),
                balance_p10: None,
                balance_p90: None,
            })
            .collect(),
    }))
}

async fn latest_baseline_computation(repo: &ForecastRepository) -> Result<uuid::Uuid, StatusCode> {
    repo.latest_successful_by_kind("baseline")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|r| r.id)
        .ok_or(StatusCode::NOT_FOUND)
}

fn balance_on(series: &[crate::forecast::types::DailyPoint], date: NaiveDate) -> f64 {
    series
        .iter()
        .find(|p| p.date == date)
        .or_else(|| series.iter().filter(|p| p.date <= date).last())
        .map(|p| p.balance)
        .unwrap_or(0.0)
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
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
    let day = date.day().min(days_in_month(y, m as u32));
    NaiveDate::from_ymd_opt(y, m as u32, day).unwrap()
}

fn derive_ml_skipped_reason(
    baseline_meta: &serde_json::Value,
    ml_enabled: bool,
) -> Option<String> {
    baseline_meta
        .get("ml_skipped_reason")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            if !ml_enabled {
                Some("sidecar_disabled".into())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::derive_ml_skipped_reason;
    use serde_json::json;

    #[test]
    fn derive_sidecar_disabled_when_ml_off_and_metadata_null() {
        let meta = json!({});
        assert_eq!(
            derive_ml_skipped_reason(&meta, false).as_deref(),
            Some("sidecar_disabled")
        );
    }

    #[test]
    fn derive_none_when_ml_enabled_and_metadata_null() {
        let meta = json!({});
        assert!(derive_ml_skipped_reason(&meta, true).is_none());
    }

    #[test]
    fn derive_preserves_existing_skip_reason() {
        let meta = json!({ "ml_skipped_reason": "insufficient_history" });
        assert_eq!(
            derive_ml_skipped_reason(&meta, false).as_deref(),
            Some("insufficient_history")
        );
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}
