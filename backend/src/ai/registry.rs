use std::sync::Arc;

use serde_json::Value;

use super::tools::{
    GetBudgetStatusTool, GetForecastTool, GetPortfolioTool, GetSubscriptionsTool,
    GetTransactionsTool, SimulatePlanTool,
};
use super::tools::AiTool;
use super::types::{ToolContext, ToolError};

pub const TOOL_NAMES: [&str; 6] = [
    "get_transactions",
    "get_subscriptions",
    "get_forecast",
    "get_budget_status",
    "get_portfolio",
    "simulate_plan",
];

pub struct ToolRegistry {
    tools: Vec<Arc<dyn AiTool>>,
}

impl ToolRegistry {
    pub fn build() -> Self {
        let tools: Vec<Arc<dyn AiTool>> = vec![
            Arc::new(GetTransactionsTool),
            Arc::new(GetSubscriptionsTool),
            Arc::new(GetForecastTool),
            Arc::new(GetBudgetStatusTool),
            Arc::new(GetPortfolioTool),
            Arc::new(SimulatePlanTool),
        ];
        Self { tools }
    }

    pub fn tools(&self) -> &[Arc<dyn AiTool>] {
        &self.tools
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn AiTool>> {
        self.tools.iter().find(|t| t.name() == name)
    }

    pub fn build_openai_tools(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.parameters_schema()
                    }
                })
            })
            .collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        ctx: &ToolContext,
        args: Value,
    ) -> Result<Value, ToolError> {
        let tool = self
            .get(name)
            .ok_or_else(|| ToolError::InvalidArgs(format!("unknown tool: {name}")))?;
        tool.execute(ctx, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_six_tools_matching_migration() {
        let reg = ToolRegistry::build();
        assert_eq!(reg.tools().len(), 6);
        for name in TOOL_NAMES {
            assert!(reg.get(name).is_some(), "missing tool {name}");
        }
    }
}
