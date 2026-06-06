use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use super::AiTool;
use crate::ai::types::{truncate_tool_result, ToolContext, ToolError};

pub struct GetPortfolioTool;

#[derive(Deserialize)]
struct Args {
    #[serde(default)]
    include_history: Option<bool>,
}

#[async_trait]
impl AiTool for GetPortfolioTool {
    fn name(&self) -> &'static str {
        "get_portfolio"
    }

    fn description(&self) -> &'static str {
        "Net worth breakdown from mirror accounts and crypto exchanges with optional 90-day history."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_history": { "type": "boolean" }
            }
        })
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        let exchange_repo = ctx.exchange_repo.as_ref().map(|r| r.as_ref());
        let breakdown = ctx
            .wealth
            .compute_extended(exchange_repo, ctx.portfolio.as_ref(), None)
            .await
            .map_err(|e| ToolError::Service(e.to_string()))?;

        let mut out = ctx.wealth.portfolio_summary_for_ai(&breakdown);
        out["accounts"] = serde_json::to_value(&breakdown.firefly.accounts)
            .map_err(|e| ToolError::Service(e.to_string()))?;

        if args.include_history.unwrap_or(false) {
            let history = ctx
                .wealth
                .history(90)
                .await
                .map_err(|e| ToolError::Service(e.to_string()))?;
            out["history"] = serde_json::to_value(history)
                .map_err(|e| ToolError::Service(e.to_string()))?;
        }

        Ok(truncate_tool_result(out, 8192))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::truncate_tool_result;

    #[test]
    fn portfolio_payload_within_8kb_cap() {
        let holdings: Vec<serde_json::Value> = (0..20)
            .map(|i| {
                serde_json::json!({
                    "exchange_id": "binance",
                    "asset": format!("TOKEN{i}"),
                    "quantity": 1000.0,
                    "value_eur": 5000.0,
                })
            })
            .collect();
        let payload = serde_json::json!({
            "total_eur": 50000.0,
            "crypto_top_holdings": holdings,
        });
        let truncated = truncate_tool_result(payload, 8192);
        let size = serde_json::to_string(&truncated).unwrap().len();
        assert!(size <= 8192);
    }
}
