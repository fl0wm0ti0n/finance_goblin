use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub firefly_id: String,
    pub account_id: Option<String>,
    pub date: NaiveDate,
    pub amount: f64,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecurringPattern {
    pub description: String,
    pub amount: f64,
    pub interval_days: i64,
    pub category_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DailyPoint {
    pub date: NaiveDate,
    pub balance: f64,
}

#[derive(Debug, Clone)]
pub struct DailyPointWithBands {
    pub date: NaiveDate,
    pub balance: f64,
    pub balance_p10: Option<f64>,
    pub balance_p90: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PortfolioWeeklyPoint {
    pub ts: NaiveDate,
    pub value_eur: f64,
    pub value_p10: Option<f64>,
    pub value_p90: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct MonthlyCashflow {
    pub month: NaiveDate,
    pub income: f64,
    pub fixed_costs: f64,
    pub variable_costs: f64,
    pub free_cashflow: f64,
}

#[derive(Debug, Clone)]
pub struct Milestones {
    pub tomorrow: f64,
    pub next_week: f64,
    pub month_end: f64,
}

#[derive(Debug, Clone)]
pub struct ProjectionResult {
    pub daily: Vec<DailyPoint>,
    pub monthly: Vec<MonthlyCashflow>,
    pub milestones: Milestones,
    pub low_confidence: bool,
    pub horizon_balances: std::collections::HashMap<i64, f64>,
}

pub fn fmt_amount(value: f64) -> String {
    format!("{:.2}", value)
}

pub fn is_transfer(payload: &Value) -> bool {
    payload
        .get("attributes")
        .and_then(|a| a.get("type"))
        .and_then(|t| t.as_str())
        .map(|t| t.eq_ignore_ascii_case("transfer"))
        .unwrap_or(false)
}
