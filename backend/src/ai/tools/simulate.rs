use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};

pub struct SimulatePlanTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    plan_id: Option<String>,
    #[serde(default)]
    version_number: Option<u32>,
}

#[async_trait]
impl AiTool for SimulatePlanTool {
    fn name(&self) -> &'static str {
        "simulate_plan"
    }

    fn description(&self) -> &'static str {
        "Read-only plan projection: plan_id/version, template, or active plan."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "template": { "type": "string", "enum": ["current", "leasing", "savings_mode", "house_purchase"] },
                "plan_id": { "type": "string" },
                "version_number": { "type": "integer" },
                "adjustments": { "type": "array" }
            }
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        if let Some(plan_id) = args.plan_id {
            let id = Uuid::parse_str(&plan_id).map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
            let projection = ctx
                .plans
                .project_readonly(id, args.version_number)
                .await
                .map_err(|e| ToolError::Service(e.to_string()))?;
            return serde_json::to_value(projection).map_err(|e| ToolError::Service(e.to_string()));
        }

        if let Some(template) = args.template {
            let projection = ctx
                .plans
                .project_ephemeral_from_template(&template, None)
                .await
                .map_err(|e| ToolError::Service(e.to_string()))?;
            return serde_json::to_value(projection).map_err(|e| ToolError::Service(e.to_string()));
        }

        if let Ok(projection) = ctx.plans.project_active_readonly().await {
            return serde_json::to_value(projection).map_err(|e| ToolError::Service(e.to_string()));
        }

        Ok(json!({ "error": "no_plan_context" }))
    }
}
