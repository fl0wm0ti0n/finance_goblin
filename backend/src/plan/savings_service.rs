use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate, Utc};

use crate::forecast::categories::{resolve_bucket, Bucket};
use crate::forecast::repository::ForecastRepository;
use crate::transactions::repository::TransactionsRepository;
use crate::transactions::types::CategoryAggregate;

use super::repository::PlanRepository;
use super::types::{
    AdjustmentDirection, AdjustmentTarget, CategorySavingsMeta, CategorySavingsResponse,
    CategorySavingsSuggestion, PlanAdjustment, fmt_amount,
};

const MIN_AVG_MONTHLY_EUR: f64 = 20.0;
const DEFAULT_REDUCTION_FRACTION: f64 = 0.5;

pub async fn category_savings_suggestions(
    plan_repo: &PlanRepository,
    tx_repo: &TransactionsRepository,
    forecast_repo: &ForecastRepository,
    plan_id: uuid::Uuid,
    months: u32,
    limit: u32,
) -> Result<CategorySavingsResponse, super::service::PlanError> {
    let _plan = plan_repo.get_plan(plan_id).await?;
    let versions = plan_repo.list_versions(plan_id).await?;
    let latest = versions
        .into_iter()
        .find(|v| v.is_latest)
        .ok_or(super::service::PlanError::VersionNotFound)?;
    let adjustments = plan_repo.load_adjustments(latest.id).await?;

    let today = Utc::now().date_naive();
    let period_end = today;
    let period_start = month_start_n_months_ago(today, months);

    let aggregates = tx_repo
        .aggregates_by_category(period_start, period_end, None)
        .await
        .map_err(super::service::PlanError::Db)?;

    let category_names = forecast_repo.category_name_map().await?;
    let forecast_config = forecast_repo.config().clone();
    let existing = existing_category_removals(&adjustments);

    let mut ranked: Vec<CategorySavingsSuggestion> = aggregates
        .into_iter()
        .filter_map(|row| map_suggestion(row, months, &category_names, &forecast_config, &existing))
        .collect();

    ranked.sort_by(|a, b| {
        b.avg_monthly_outflow_eur
            .parse::<f64>()
            .unwrap_or(0.0)
            .partial_cmp(&a.avg_monthly_outflow_eur.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.category_id.cmp(&b.category_id))
    });
    ranked.truncate(limit as usize);

    Ok(CategorySavingsResponse {
        suggestions: ranked,
        meta: CategorySavingsMeta {
            months,
            limit,
            ranking: "deterministic_aggregate",
        },
    })
}

fn existing_category_removals(adjustments: &[PlanAdjustment]) -> HashSet<String> {
    adjustments
        .iter()
        .filter(|a| {
            a.target_type == AdjustmentTarget::Category
                && a.direction == AdjustmentDirection::RemoveOutflow
        })
        .filter_map(|a| a.target_key.clone())
        .collect()
}

fn map_suggestion(
    row: CategoryAggregate,
    months: u32,
    category_names: &HashMap<String, String>,
    forecast_config: &crate::config::ForecastConfig,
    existing: &HashSet<String>,
) -> Option<CategorySavingsSuggestion> {
    let category_id = row.category_id?;
    if existing.contains(&category_id) {
        return None;
    }
    let avg_monthly = row.total_outflow / months as f64;
    if avg_monthly < MIN_AVG_MONTHLY_EUR {
        return None;
    }
    if resolve_bucket(Some(&category_id), category_names, forecast_config) == Bucket::Fixed {
        return None;
    }
    let name = row
        .category_name
        .unwrap_or_else(|| category_names.get(&category_id).cloned().unwrap_or(category_id.clone()));
    let suggested = avg_monthly * DEFAULT_REDUCTION_FRACTION;
    Some(CategorySavingsSuggestion {
        category_id: category_id.clone(),
        category_name: name.clone(),
        avg_monthly_outflow_eur: fmt_amount(avg_monthly),
        transaction_count: row.transaction_count,
        suggested_reduction_eur: fmt_amount(suggested),
        evidence_summary: format!(
            "{months}-month avg €{avg:.2}/mo; {} transactions",
            row.transaction_count,
            avg = avg_monthly
        ),
    })
}

fn month_start_n_months_ago(today: NaiveDate, months: u32) -> NaiveDate {
    let total = today.year() as i32 * 12 + today.month() as i32 - 1 - months as i32 + 1;
    let y = total.div_euclid(12);
    let m = (total.rem_euclid(12) + 1) as u32;
    NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(today)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::plan::types::{AdjustmentFrequency, PlanAdjustment};

    fn test_forecast_config() -> crate::config::ForecastConfig {
        crate::config::ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        }
    }

    fn sample_adjustment(category_id: &str) -> PlanAdjustment {
        PlanAdjustment {
            id: Uuid::new_v4(),
            version_id: Uuid::new_v4(),
            direction: AdjustmentDirection::RemoveOutflow,
            amount: 50.0,
            frequency: AdjustmentFrequency::Monthly,
            target_type: AdjustmentTarget::Category,
            target_key: Some(category_id.into()),
            label: None,
            effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_to: None,
            sort_order: 0,
        }
    }

    #[test]
    fn excludes_existing_category_removals() {
        let adj = vec![sample_adjustment("42")];
        let existing = existing_category_removals(&adj);
        assert!(existing.contains("42"));
    }

    #[test]
    fn map_suggestion_skips_below_min_spend() {
        let row = CategoryAggregate {
            category_id: Some("9".into()),
            category_name: Some("Coffee".into()),
            total_outflow: 30.0,
            total_inflow: 0.0,
            transaction_count: 3,
        };
        let names = HashMap::new();
        let cfg = test_forecast_config();
        assert!(map_suggestion(row, 6, &names, &cfg, &HashSet::new()).is_none());
    }

    #[test]
    fn map_suggestion_ranks_entertainment() {
        let row = CategoryAggregate {
            category_id: Some("42".into()),
            category_name: Some("Entertainment".into()),
            total_outflow: 513.0,
            total_inflow: 0.0,
            transaction_count: 24,
        };
        let names = HashMap::new();
        let cfg = test_forecast_config();
        let s = map_suggestion(row, 6, &names, &cfg, &HashSet::new()).unwrap();
        assert_eq!(s.suggested_reduction_eur, "42.75");
        assert!(s.evidence_summary.contains("24 transactions"));
    }
}
