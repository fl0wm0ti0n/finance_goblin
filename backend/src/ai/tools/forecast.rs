use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use super::AiTool;
use crate::ai::types::{ToolContext, ToolError};

pub struct GetForecastTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default = "default_horizon")]
    horizon: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    variant: Option<String>,
}

fn default_horizon() -> String {
    "3m".into()
}

#[async_trait]
impl AiTool for GetForecastTool {
    fn name(&self) -> &'static str {
        "get_forecast"
    }

    fn description(&self) -> &'static str {
        "Return latest forecast computation summary for a horizon."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "horizon": { "type": "string", "enum": ["3m", "6m", "12m", "24m"] },
                "account_id": { "type": "string" },
                "variant": { "type": "string", "enum": ["baseline", "ml_enhanced"], "description": "Forecast variant; default baseline" }
            }
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let summary = ctx
            .forecast
            .forecast_summary_for_tool(args.horizon, args.account_id, args.variant)
            .await
            .map_err(|e| ToolError::Service(e.to_string()))?;
        Ok(summary)
    }
}
