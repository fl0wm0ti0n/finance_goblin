use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use std::sync::Arc;

use crate::exchanges::repository::ExchangeRepository;
use crate::portfolio::PortfolioEngine;
use crate::alerts::AlertService;
use crate::config::PrivacyConfig;
use crate::forecast::ForecastService;
use crate::plan::PlanService;
use crate::subscriptions::SubscriptionService;
use crate::transactions::TransactionsService;
use crate::wealth::WealthService;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("service error: {0}")]
    Service(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub struct ToolContext {
    pub transactions: TransactionsService,
    pub subscriptions: SubscriptionService,
    pub forecast: ForecastService,
    pub plans: PlanService,
    pub wealth: WealthService,
    pub alerts: AlertService,
    pub exchange_repo: Option<Arc<ExchangeRepository>>,
    pub portfolio: Option<PortfolioEngine>,
    pub privacy: PrivacyConfig,
    pub user_subject: String,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SseTokenEvent {
    pub delta: String,
}

#[derive(Debug, Serialize)]
pub struct SseToolStartEvent {
    pub tool: String,
    pub call_id: String,
}

#[derive(Debug, Serialize)]
pub struct SseToolEndEvent {
    pub tool: String,
    pub duration_ms: u64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SseDoneEvent {
    pub message_id: String,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SseErrorEvent {
    pub code: String,
    pub message: String,
}

pub fn truncate_tool_result(value: Value, max_bytes: usize) -> Value {
    let serialized = serde_json::to_string(&value).unwrap_or_default();
    if serialized.len() <= max_bytes {
        return downsample_series(value, 30);
    }

    let summary = summarize_for_truncation(&value);
    serde_json::json!({
        "truncated": true,
        "summary": summary,
    })
}

fn downsample_series(mut value: Value, max_points: usize) -> Value {
    if let Value::Object(ref mut map) = value {
        for key in ["series", "history", "daily", "rows"] {
            if let Some(Value::Array(arr)) = map.get(key).cloned() {
                if arr.len() > max_points {
                    let sampled = downsample_array(&arr, max_points);
                    map.insert(key.to_string(), Value::Array(sampled));
                }
            }
        }
    }
    value
}

fn downsample_array(arr: &[Value], max_points: usize) -> Vec<Value> {
    if arr.len() <= max_points {
        return arr.to_vec();
    }
    let step = (arr.len() as f64 / max_points as f64).ceil() as usize;
    arr.iter()
        .enumerate()
        .filter(|(i, _)| i % step == 0)
        .map(|(_, v)| v.clone())
        .collect()
}

fn summarize_for_truncation(value: &Value) -> Value {
    let mut summary = serde_json::Map::new();
    if let Some(n) = value.get("total").and_then(|v| v.as_f64()) {
        summary.insert("total".into(), Value::from(n));
    }
    if let Some(arr) = value.as_array() {
        let nums: Vec<f64> = arr
            .iter()
            .filter_map(|v| v.get("balance").or_else(|| v.get("amount")).and_then(|x| x.as_f64()))
            .collect();
        if !nums.is_empty() {
            summary.insert("min".into(), nums.iter().cloned().fold(f64::INFINITY, f64::min).into());
            summary.insert(
                "max".into(),
                nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max).into(),
            );
            summary.insert("latest".into(), nums.last().copied().unwrap_or(0.0).into());
        }
    }
    Value::Object(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_sets_flag() {
        let big = serde_json::json!({ "data": "x".repeat(10_000) });
        let out = truncate_tool_result(big, 100);
        assert_eq!(out["truncated"], true);
    }

    #[test]
    fn downsample_reduces_series() {
        let arr: Vec<Value> = (0..100).map(|i| json_number(i)).collect();
        let input = serde_json::json!({ "series": arr });
        let out = downsample_series(input, 30);
        let series = out["series"].as_array().unwrap();
        assert!(series.len() <= 30);
    }

    fn json_number(i: i32) -> Value {
        serde_json::json!({ "balance": i as f64 })
    }
}
