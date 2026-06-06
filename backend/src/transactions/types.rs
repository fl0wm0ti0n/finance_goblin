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
