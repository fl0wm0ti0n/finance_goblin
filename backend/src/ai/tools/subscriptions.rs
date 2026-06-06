use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};

pub struct GetSubscriptionsTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    include_price_events: Option<bool>,
}

fn is_privacy_hash_filter(value: &str) -> bool {
    value.starts_with("Counterparty-")
}

fn reject_privacy_hash_filters(status: Option<&str>, kind: Option<&str>) -> Result<(), ToolError> {
    if status.is_some_and(is_privacy_hash_filter) || kind.is_some_and(is_privacy_hash_filter) {
        return Err(ToolError::InvalidArgs(
            "privacy hash is not a valid filter".into(),
        ));
    }
    Ok(())
}

fn collect_merchant_names(patterns: &[Value]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for p in patterns {
        if let Some(name) = p.get("display_name").and_then(|v| v.as_str()) {
            if seen.insert(name.to_string()) {
                names.push(name.to_string());
            }
        }
    }
    names
}

#[async_trait]
impl AiTool for GetSubscriptionsTool {
    fn name(&self) -> &'static str {
        "get_subscriptions"
    }

    fn description(&self) -> &'static str {
        "List detected subscription and standing-order patterns. When the user asks to list services, enumerate all display_name values from results with amounts. Use kind: subscription for discretionary subs; omit kind for all patterns."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["pending", "confirmed", "rejected"],
                    "description": "Filter by pattern status. Must be a documented enum value — never use privacy hashes (Counterparty-*)."
                },
                "kind": {
                    "type": "string",
                    "enum": ["subscription", "standing_order"],
                    "description": "Filter by pattern kind. Must be subscription or standing_order — never use privacy hashes (Counterparty-*)."
                },
                "include_price_events": { "type": "boolean" }
            }
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        reject_privacy_hash_filters(args.status.as_deref(), args.kind.as_deref())?;

        let patterns = ctx
            .subscriptions
            .list_patterns(args.status.as_deref(), args.kind.as_deref())
            .await
            .map_err(|e| ToolError::Service(e.to_string()))?;

        let patterns_value = serde_json::to_value(&patterns)
            .map_err(|e| ToolError::Service(e.to_string()))?;
        let patterns_array = patterns_value
            .as_array()
            .cloned()
            .unwrap_or_default();
        let merchant_names = collect_merchant_names(&patterns_array);
        let patterns_count = patterns_array.len() as i64;

        let mut out = json!({
            "patterns": patterns,
            "patterns_count": patterns_count,
            "merchant_names": merchant_names,
        });
        if args.include_price_events.unwrap_or(false) {
            let events = ctx
                .subscriptions
                .recent_price_events(20)
                .await
                .map_err(|e| ToolError::Service(e.to_string()))?;
            out["price_events"] = json!(events);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_hash_filter_rejected() {
        assert!(is_privacy_hash_filter("Counterparty-abc123"));
        assert!(!is_privacy_hash_filter("confirmed"));
        assert!(reject_privacy_hash_filters(Some("Counterparty-x"), None).is_err());
        assert!(reject_privacy_hash_filters(None, Some("Counterparty-y")).is_err());
        assert!(reject_privacy_hash_filters(Some("confirmed"), Some("subscription")).is_ok());
    }

    #[test]
    fn merchant_names_deduped_in_order() {
        let patterns = vec![
            json!({"display_name": "Netflix"}),
            json!({"display_name": "Spotify"}),
            json!({"display_name": "Netflix"}),
        ];
        assert_eq!(
            collect_merchant_names(&patterns),
            vec!["Netflix", "Spotify"]
        );
    }

    #[test]
    fn schema_includes_kind_enum() {
        let tool = GetSubscriptionsTool;
        let schema = tool.parameters_schema();
        let kinds = schema["properties"]["kind"]["enum"].as_array().unwrap();
        assert!(kinds.contains(&json!("subscription")));
        assert!(kinds.contains(&json!("standing_order")));
    }
}
