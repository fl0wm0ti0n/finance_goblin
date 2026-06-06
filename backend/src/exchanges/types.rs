use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExchangeSyncState {
    pub last_trade_time: Option<DateTime<Utc>>,
    pub last_transfer_time: Option<DateTime<Utc>>,
    pub last_funding_time: Option<DateTime<Utc>>,
    pub active_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTest {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
    pub withdraw_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeHolding {
    pub asset: String,
    pub quantity: f64,
    pub product_type: String,
    pub market_value_usd: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    pub avg_cost: Option<f64>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTrade {
    pub external_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub fee: Option<f64>,
    pub fee_asset: Option<String>,
    pub realized_pnl: Option<f64>,
    pub executed_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTransfer {
    pub external_id: String,
    pub transfer_type: String,
    pub asset: String,
    pub quantity: f64,
    pub status: String,
    pub executed_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeFundingEvent {
    pub external_id: String,
    pub symbol: Option<String>,
    pub amount: f64,
    pub asset: String,
    pub event_type: String,
    pub executed_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ExchangeError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("not configured")]
    NotConfigured,
    #[error("method not allowed: {0}")]
    MethodNotAllowed(String),
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ConnectionRow {
    pub id: String,
    pub enabled: bool,
    pub connection_state: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExchangeListItem {
    pub id: String,
    pub enabled: bool,
    pub connection_state: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub counts: ExchangeEntityCounts,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ExchangeEntityCounts {
    pub holdings: i64,
    pub trades: i64,
    pub transfers: i64,
    pub funding: i64,
}
