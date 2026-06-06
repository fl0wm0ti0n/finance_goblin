use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    Scarcity,
    BudgetDrift,
    PlanViability,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scarcity => "scarcity",
            Self::BudgetDrift => "budget_drift",
            Self::PlanViability => "plan_viability",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scarcity" => Some(Self::Scarcity),
            "budget_drift" => Some(Self::BudgetDrift),
            "plan_viability" => Some(Self::PlanViability),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Dismissed,
    Resolved,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Acknowledged => "acknowledged",
            Self::Dismissed => "dismissed",
            Self::Resolved => "resolved",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "acknowledged" => Some(Self::Acknowledged),
            "dismissed" => Some(Self::Dismissed),
            "resolved" => Some(Self::Resolved),
            _ => None,
        }
    }
}

pub use crate::config::AlertsConfig;

#[derive(Debug, Clone)]
pub struct EvalContext {
    pub forecast_computation_id: Uuid,
    pub plan_computation_id: Option<Uuid>,
    pub config: AlertsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCandidate {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub fingerprint: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRow {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub status: String,
    pub fingerprint: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub context: serde_json::Value,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvalResult {
    pub created: u32,
    pub resolved: u32,
    pub updated: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AlertListFilter {
    pub status: Option<String>,
    pub include_dismissed: bool,
    pub limit: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("alert not found")]
    NotFound,
    #[error("invalid status transition")]
    InvalidTransition,
}
