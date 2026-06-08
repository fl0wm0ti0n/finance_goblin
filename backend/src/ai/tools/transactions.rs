use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::transactions::{AggregateFilter, GroupBy};

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};

pub struct GetTransactionsTool;

#[derive(Deserialize)]
struct Args {
    period_start: String,
    period_end: String,
    #[serde(default)]
    category_id: Option<String>,
    #[serde(default)]
    category_search: Option<String>,
    #[serde(default = "default_group_by")]
    group_by: String,
}

fn default_group_by() -> String {
    "category".into()
}

#[async_trait]
impl AiTool for GetTransactionsTool {
    fn name(&self) -> &'static str {
        "get_transactions"
    }

    fn description(&self) -> &'static str {
        "Aggregate mirror transactions by category or month for a date period. Returns period totals, period_status, mirror_date_bounds, category_matches (when searching), and grouped buckets."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "period_start": { "type": "string", "description": "YYYY-MM-DD" },
                "period_end": { "type": "string", "description": "YYYY-MM-DD" },
                "category_id": {
                    "type": "string",
                    "description": "Firefly category ID (numeric string). For keyword search use category_search instead."
                },
                "category_search": {
                    "type": "string",
                    "description": "Keyword to match category names (case-insensitive, min 2 chars). Example: 'amazon', 'strom'."
                },
                "group_by": { "type": "string", "enum": ["category", "month"] }
            },
            "required": ["period_start", "period_end"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let start = NaiveDate::parse_from_str(&args.period_start, "%Y-%m-%d")
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let end = NaiveDate::parse_from_str(&args.period_end, "%Y-%m-%d")
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let group_by = match args.group_by.as_str() {
            "month" => GroupBy::Month,
            _ => GroupBy::Category,
        };
        let agg = ctx
            .transactions
            .aggregates(AggregateFilter {
                period_start: start,
                period_end: end,
                category_id: args.category_id,
                category_search: args.category_search,
                group_by,
            })
            .await
            .map_err(|e| match e {
                crate::transactions::TransactionsError::InvalidArgs(msg) => {
                    ToolError::InvalidArgs(msg)
                }
                crate::transactions::TransactionsError::NotFound(msg) => {
                    ToolError::InvalidArgs(format!("category not found: {msg}"))
                }
                crate::transactions::TransactionsError::Db(msg) => {
                    ToolError::Service(msg.to_string())
                }
            })?;
        serde_json::to_value(agg).map_err(|e| ToolError::Service(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_includes_category_search_and_descriptions() {
        let tool = GetTransactionsTool;
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["category_search"].is_object());
        assert!(schema["properties"]["category_id"]["description"]
            .as_str()
            .unwrap()
            .contains("category_search"));
    }
}
