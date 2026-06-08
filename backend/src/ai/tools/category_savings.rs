use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};
use crate::plan::savings_service;
use crate::transactions::repository::TransactionsRepository;

pub struct GetCategorySavingsTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default)]
    plan_id: Option<String>,
    #[serde(default)]
    months: Option<u32>,
    #[serde(default)]
    limit: Option<u32>,
}

#[async_trait]
impl AiTool for GetCategorySavingsTool {
    fn name(&self) -> &'static str {
        "get_category_savings"
    }

    fn description(&self) -> &'static str {
        "Rank reducible expense categories for a goal plan using aggregate mirror data only. Returns the same deterministic ranking as GET category-savings-suggestions — no raw transactions."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "plan_id": { "type": "string", "description": "Goal plan UUID" },
                "months": { "type": "integer", "description": "Lookback months (default 6, max 24)" },
                "limit": { "type": "integer", "description": "Max suggestions (default 10)" }
            },
            "required": ["plan_id"]
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let plan_id = args
            .plan_id
            .ok_or_else(|| ToolError::InvalidArgs("plan_id is required".into()))?;
        let plan_id = Uuid::parse_str(&plan_id)
            .map_err(|_| ToolError::InvalidArgs("invalid plan_id".into()))?;
        let months = args.months.unwrap_or(6).clamp(1, 24);
        let limit = args.limit.unwrap_or(10).clamp(1, 50);

        let tx_repo = TransactionsRepository::new(ctx.plans.repository().pool().clone());
        let response = savings_service::category_savings_suggestions(
            ctx.plans.repository(),
            &tx_repo,
            ctx.forecast.repository(),
            plan_id,
            months,
            limit,
        )
        .await
        .map_err(|e| ToolError::Service(e.to_string()))?;

        Ok(serde_json::to_value(response).map_err(|e| ToolError::Service(e.to_string()))?)
    }
}
