use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};

pub struct GetBudgetStatusTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default)]
    category_id: Option<String>,
}

#[async_trait]
impl AiTool for GetBudgetStatusTool {
    fn name(&self) -> &'static str {
        "get_budget_status"
    }

    fn description(&self) -> &'static str {
        "MTD budget status vs active plan category targets (aligned with alert budget drift)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category_id": { "type": "string" }
            }
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let status = ctx
            .alerts
            .budget_status_for_tool(args.category_id)
            .await
            .map_err(|e| ToolError::Service(e.to_string()))?;
        Ok(status)
    }
}
