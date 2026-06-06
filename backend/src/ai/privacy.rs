use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::config::PrivacyConfig;

pub struct PrivacyLayer {
    config: PrivacyConfig,
    pepper: String,
}

impl PrivacyLayer {
    pub fn new(config: PrivacyConfig) -> Self {
        let pepper = std::env::var("PRIVACY_PEPPER")
            .unwrap_or_else(|_| "flow-finance-ai-default-pepper".into());
        Self { config, pepper }
    }

    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }

    pub fn redact_tool_result(&self, tool_name: &str, value: Value) -> Value {
        if tool_name == "get_transactions" && !self.config.allow_raw_transactions {
            if contains_row_array(&value) {
                return json!({
                    "error": "raw_transactions_disabled",
                    "hint": "use category aggregates"
                });
            }
        }
        self.walk_value_for_tool(tool_name, value, false)
    }

    pub fn summarize_args(&self, args: &Value) -> Value {
        self.walk_value_for_tool("", args.clone(), false)
    }

    fn is_subscription_label_field(tool_name: &str, key: &str) -> bool {
        tool_name == "get_subscriptions" && matches!(key, "display_name" | "merchant_names")
    }

    fn walk_value_for_tool(
        &self,
        tool_name: &str,
        value: Value,
        preserve_label_strings: bool,
    ) -> Value {
        match value {
            Value::Object(mut map) => {
                let keys: Vec<String> = map.keys().cloned().collect();
                for key in keys {
                    if let Some(v) = map.remove(&key) {
                        let child_preserve = preserve_label_strings
                            || Self::is_subscription_label_field(tool_name, &key);
                        let redacted = if self.is_sensitive_field(&key) {
                            self.redact_field(tool_name, &key, v)
                        } else {
                            self.walk_value_for_tool(tool_name, v, child_preserve)
                        };
                        map.insert(key, redacted);
                    }
                }
                Value::Object(map)
            }
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| self.walk_value_for_tool(tool_name, v, preserve_label_strings))
                    .collect(),
            ),
            Value::String(s) => {
                if preserve_label_strings {
                    Value::String(self.redact_iban_only(&s))
                } else {
                    Value::String(self.redact_string(&s))
                }
            }
            other => other,
        }
    }

    fn is_sensitive_field(&self, key: &str) -> bool {
        matches!(
            key,
            "iban" | "payee" | "description" | "counterparty" | "destination_name"
        )
    }

    fn redact_field(&self, tool_name: &str, key: &str, value: Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.redact_named_field(key, &s)),
            other => self.walk_value_for_tool(tool_name, other, false),
        }
    }

    fn redact_named_field(&self, key: &str, s: &str) -> String {
        if key == "iban" && self.config.redact_iban {
            return "[IBAN_REDACTED]".into();
        }
        if self.config.redact_counterparties
            && matches!(key, "payee" | "description" | "counterparty" | "destination_name")
        {
            return self.hash_counterparty(s);
        }
        self.redact_string(s)
    }

    fn redact_iban_only(&self, s: &str) -> String {
        if !self.config.redact_iban {
            return s.to_string();
        }
        static IBAN_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let re = IBAN_RE.get_or_init(|| {
            Regex::new(r"\b[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}\b").expect("iban regex")
        });
        re.replace_all(s, "[IBAN_REDACTED]").to_string()
    }

    fn redact_string(&self, s: &str) -> String {
        let out = self.redact_iban_only(s);
        if self.config.redact_counterparties && out.len() > 3 && !out.starts_with("Counterparty-") {
            // leave numeric/short strings
            if out.chars().any(|c| c.is_alphabetic()) && out.len() > 8 {
                return self.hash_counterparty(&out);
            }
        }
        out
    }

    fn hash_counterparty(&self, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.pepper.as_bytes());
        hasher.update(value.as_bytes());
        let digest = hasher.finalize();
        let hex = format!("{:x}", digest);
        format!("Counterparty-{}", &hex[..8])
    }
}

fn contains_row_array(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            if map.contains_key("raw_rows") {
                return map.get("raw_rows").and_then(|v| v.as_array()).is_some_and(|a| !a.is_empty());
            }
            map.values().any(contains_row_array)
        }
        Value::Array(arr) => arr.iter().any(contains_row_array),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PrivacyConfig;

    fn layer() -> PrivacyLayer {
        PrivacyLayer::new(PrivacyConfig {
            allow_raw_transactions: false,
            redact_iban: true,
            redact_counterparties: true,
        })
    }

    #[test]
    fn redacts_iban_in_nested_json() {
        let layer = layer();
        let input = json!({
            "account": { "iban": "DE89370400440532013000" },
            "note": "paid DE89370400440532013000"
        });
        let out = layer.redact_tool_result("get_portfolio", input);
        assert!(out.to_string().contains("[IBAN_REDACTED]"));
        assert!(!out.to_string().contains("DE89370400440532013000"));
    }

    #[test]
    fn counterparty_hash_is_stable() {
        let layer = layer();
        let a = layer.redact_tool_result(
            "get_transactions",
            json!({ "payee": "Netflix GmbH" }),
        );
        let b = layer.redact_tool_result(
            "get_transactions",
            json!({ "payee": "Netflix GmbH" }),
        );
        assert_eq!(a["payee"], b["payee"]);
        assert!(a["payee"].as_str().unwrap().starts_with("Counterparty-"));
    }

    #[test]
    fn blocks_raw_rows_when_disabled() {
        let layer = layer();
        let input = json!({
            "raw_rows": [{ "description": "coffee", "amount": -5.0 }]
        });
        let out = layer.redact_tool_result("get_transactions", input);
        assert_eq!(out["error"], "raw_transactions_disabled");
    }

    #[test]
    fn summarize_args_strips_pii() {
        let layer = layer();
        let args = json!({ "payee": "Amazon", "period": "2026-05" });
        let summary = layer.summarize_args(&args);
        assert!(summary.get("period").is_some());
    }

    #[test]
    fn get_subscriptions_preserves_display_name_and_merchant_names() {
        let layer = layer();
        let input = json!({
            "patterns": [
                { "display_name": "YouTube Premium", "amount": -12.99 },
                { "display_name": "Cursor Pro", "amount": -20.0 }
            ],
            "patterns_count": 2,
            "merchant_names": ["YouTube Premium", "Cursor Pro"]
        });
        let out = layer.redact_tool_result("get_subscriptions", input);
        assert_eq!(out["patterns"][0]["display_name"], "YouTube Premium");
        assert_eq!(out["patterns"][1]["display_name"], "Cursor Pro");
        assert_eq!(out["merchant_names"][0], "YouTube Premium");
        assert_eq!(out["merchant_names"][1], "Cursor Pro");
    }

    #[test]
    fn get_subscriptions_still_redacts_other_long_strings() {
        let layer = layer();
        let input = json!({ "payee_key": "some-long-merchant-identifier-value" });
        let out = layer.redact_tool_result("get_subscriptions", input);
        assert!(out["payee_key"]
            .as_str()
            .unwrap()
            .starts_with("Counterparty-"));
    }
}
