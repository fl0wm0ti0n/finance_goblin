use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    Category,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateFilter {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub category_id: Option<String>,
    pub category_search: Option<String>,
    pub group_by: GroupBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMatch {
    pub category_id: String,
    pub category_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorDateBounds {
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAggregate {
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub total_outflow: f64,
    pub total_inflow: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthAggregate {
    pub month: String,
    pub total_outflow: f64,
    pub total_inflow: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransactionRow {
    pub date: String,
    pub amount: f64,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodStatus {
    NoRowsInPeriod,
    RowsZeroOutflow,
    RowsUncategorized,
    RowsWithOutflow,
}

pub fn compute_period_status(
    total_count: i64,
    total_outflow: f64,
    uncategorized_count: i64,
) -> PeriodStatus {
    if total_count == 0 {
        PeriodStatus::NoRowsInPeriod
    } else if total_outflow == 0.0 {
        PeriodStatus::RowsZeroOutflow
    } else if uncategorized_count == total_count {
        PeriodStatus::RowsUncategorized
    } else {
        PeriodStatus::RowsWithOutflow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodSummary {
    pub total_count: i64,
    pub total_outflow: f64,
    pub total_inflow: f64,
    pub uncategorized_count: i64,
}

/// Sentinel query token for uncategorized transactions (DEC-0087).
pub const UNCATEGORIZED_CATEGORY_ID: &str = "__uncategorized__";

pub const EXPENSE_SERIES_DEFAULT_MONTHS: u32 = 12;
pub const EXPENSE_SERIES_MAX_MONTHS: u32 = 24;
pub const CATEGORY_CATALOG_CAP: i64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpenseSeriesCategory<'a> {
    Uncategorized,
    MirrorId(&'a str),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSeriesMonth {
    pub month: String,
    pub outflow_eur: f64,
    pub inflow_eur: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSeriesSummary {
    pub mom_delta_pct: f64,
    pub best_month: String,
    pub worst_month: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCatalogItem {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCatalogResponse {
    pub categories: Vec<CategoryCatalogItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSeriesMeta {
    pub period_start: String,
    pub period_end: String,
}

pub fn validate_expense_series_months(months: u32) -> Result<(), String> {
    if months == 0 {
        return Err("months must be at least 1".into());
    }
    if months > EXPENSE_SERIES_MAX_MONTHS {
        return Err(format!(
            "months must be at most {EXPENSE_SERIES_MAX_MONTHS}"
        ));
    }
    Ok(())
}

pub fn compute_expense_series_summary(months: &[ExpenseSeriesMonth]) -> ExpenseSeriesSummary {
    let mom_delta_pct = if months.len() >= 2 {
        let prev = months[months.len() - 2].outflow_eur;
        let curr = months[months.len() - 1].outflow_eur;
        if prev == 0.0 {
            if curr == 0.0 {
                0.0
            } else {
                100.0
            }
        } else {
            ((curr - prev) / prev) * 100.0
        }
    } else {
        0.0
    };

    let best_month = months
        .iter()
        .max_by(|a, b| {
            a.outflow_eur
                .partial_cmp(&b.outflow_eur)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| m.month.clone())
        .unwrap_or_default();

    let active: Vec<&ExpenseSeriesMonth> = months
        .iter()
        .filter(|m| m.transaction_count > 0)
        .collect();
    let worst_source = if active.is_empty() {
        months.iter().collect::<Vec<_>>()
    } else {
        active
    };
    let worst_month = worst_source
        .iter()
        .min_by(|a, b| {
            a.outflow_eur
                .partial_cmp(&b.outflow_eur)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| m.month.clone())
        .unwrap_or_default();

    ExpenseSeriesSummary {
        mom_delta_pct,
        best_month,
        worst_month,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSeriesResponse {
    pub category_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncategorized: Option<bool>,
    pub months: Vec<ExpenseSeriesMonth>,
    pub summary: ExpenseSeriesSummary,
    pub meta: ExpenseSeriesMeta,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAggregates {
    pub period_start: String,
    pub period_end: String,
    pub group_by: String,
    pub total_transaction_count: i64,
    pub total_outflow: f64,
    pub total_inflow: f64,
    pub uncategorized_transaction_count: i64,
    pub period_status: PeriodStatus,
    pub mirror_date_bounds: MirrorDateBounds,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_search: Option<String>,
    pub category_matches: Vec<CategoryMatch>,
    pub category_matches_truncated: bool,
    pub search_attempted: bool,
    pub category_id_ignored: bool,
    pub by_category: Option<Vec<CategoryAggregate>>,
    pub by_month: Option<Vec<MonthAggregate>>,
    pub raw_rows: Option<Vec<RawTransactionRow>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_summary_mom_best_worst() {
        let months = vec![
            ExpenseSeriesMonth {
                month: "2026-04".into(),
                outflow_eur: 100.0,
                inflow_eur: 0.0,
                transaction_count: 2,
            },
            ExpenseSeriesMonth {
                month: "2026-05".into(),
                outflow_eur: 80.0,
                inflow_eur: 0.0,
                transaction_count: 1,
            },
            ExpenseSeriesMonth {
                month: "2026-06".into(),
                outflow_eur: 120.0,
                inflow_eur: 0.0,
                transaction_count: 3,
            },
        ];
        let summary = compute_expense_series_summary(&months);
        assert!((summary.mom_delta_pct - 50.0).abs() < 0.01);
        assert_eq!(summary.best_month, "2026-06");
        assert_eq!(summary.worst_month, "2026-05");
    }

    #[test]
    fn compute_summary_zero_spine_months() {
        let months = vec![
            ExpenseSeriesMonth {
                month: "2026-05".into(),
                outflow_eur: 0.0,
                inflow_eur: 0.0,
                transaction_count: 0,
            },
            ExpenseSeriesMonth {
                month: "2026-06".into(),
                outflow_eur: 0.0,
                inflow_eur: 0.0,
                transaction_count: 0,
            },
        ];
        let summary = compute_expense_series_summary(&months);
        assert_eq!(summary.mom_delta_pct, 0.0);
        assert!(["2026-05", "2026-06"].contains(&summary.best_month.as_str()));
        assert!(["2026-05", "2026-06"].contains(&summary.worst_month.as_str()));
    }

    #[test]
    fn uncategorized_sentinel_constant() {
        assert_eq!(UNCATEGORIZED_CATEGORY_ID, "__uncategorized__");
    }

    #[test]
    fn validate_months_cap() {
        assert!(validate_expense_series_months(24).is_ok());
        assert!(validate_expense_series_months(25).is_err());
    }
}
