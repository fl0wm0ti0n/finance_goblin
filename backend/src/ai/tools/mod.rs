mod budget;
mod forecast;
mod portfolio;
mod simulate;
mod subscriptions;
mod transactions;

pub use budget::GetBudgetStatusTool;
pub use forecast::GetForecastTool;
pub use portfolio::GetPortfolioTool;
pub use simulate::SimulatePlanTool;
pub use subscriptions::GetSubscriptionsTool;
pub use transactions::GetTransactionsTool;

use async_trait::async_trait;
use serde_json::Value;

use super::types::{ToolContext, ToolError};

#[async_trait]
pub trait AiTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError>;
}
