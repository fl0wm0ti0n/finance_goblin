use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWealthRow {
    pub firefly_id: String,
    pub name: String,
    pub account_role: Option<String>,
    pub currency: String,
    pub balance: f64,
    pub is_overdrawn: bool,
    pub pct_of_total: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireflyBreakdown {
    pub subtotal_eur: f64,
    pub mixed_currency: bool,
    pub accounts: Vec<AccountWealthRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoExchangeSummary {
    pub id: String,
    pub connection_state: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub subtotal_eur: f64,
    pub holdings_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoHoldingRow {
    pub exchange_id: String,
    pub asset: String,
    pub quantity: f64,
    pub value_eur: f64,
    pub unrealized_pnl_eur: Option<f64>,
    pub product_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsAllRow {
    pub asset: String,
    pub quantity: f64,
    pub product_type: String,
    pub value_eur: Option<f64>,
    pub unrealized_pnl_eur: Option<f64>,
    pub native_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoBreakdown {
    pub subtotal_eur: f64,
    pub fx_complete: bool,
    pub exchanges: Vec<CryptoExchangeSummary>,
    pub holdings_top: Vec<CryptoHoldingRow>,
    pub holdings_all: Vec<HoldingsAllRow>,
    pub unpriced_assets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlSummary {
    pub realized_eur: f64,
    pub unrealized_eur: f64,
    pub total_return_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationWeights {
    pub etf_traditional_pct: f64,
    pub crypto_pct: f64,
    pub cash_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationGap {
    pub current: AllocationWeights,
    pub target: AllocationWeights,
    pub gaps: AllocationWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedWealthBreakdown {
    pub reporting_currency: String,
    pub firefly: FireflyBreakdown,
    pub crypto: CryptoBreakdown,
    pub total_eur: f64,
    pub pnl: PnlSummary,
    pub fx_incomplete: bool,
    pub crypto_placeholder: bool,
    pub allocation_gap: Option<AllocationGap>,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthBreakdown {
    pub total: f64,
    pub reporting_currency: String,
    pub mixed_currency: bool,
    pub crypto_placeholder: bool,
    pub accounts: Vec<AccountWealthRow>,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub computed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firefly: Option<FireflyBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto: Option<CryptoBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnl: Option<PnlSummary>,
    #[serde(default)]
    pub fx_incomplete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_eur: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_gap: Option<AllocationGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthHistoryPoint {
    pub snapshot_date: NaiveDate,
    pub total_eur: f64,
    pub mixed_currency: bool,
    pub account_count: i32,
    #[serde(default)]
    pub crypto_value_eur: f64,
    #[serde(default)]
    pub firefly_value_eur: f64,
    #[serde(default)]
    pub total_return_pct: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AssetAccountRow {
    pub firefly_id: String,
    pub name: Option<String>,
    pub currency: Option<String>,
    pub balance: Option<f64>,
    pub account_role: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SnapshotRow {
    pub snapshot_date: NaiveDate,
    pub total_eur: f64,
    pub mixed_currency: bool,
    pub account_count: i32,
    pub payload: serde_json::Value,
    pub crypto_value_eur: Option<f64>,
    pub firefly_value_eur: Option<f64>,
    pub total_return_pct: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum WealthError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}
