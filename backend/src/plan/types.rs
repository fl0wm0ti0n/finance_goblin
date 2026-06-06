use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanTemplate {
    Current,
    Leasing,
    SavingsMode,
    HousePurchase,
    Custom,
    AllocationTarget,
}

impl PlanTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Leasing => "leasing",
            Self::SavingsMode => "savings_mode",
            Self::HousePurchase => "house_purchase",
            Self::Custom => "custom",
            Self::AllocationTarget => "allocation_target",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "current" => Some(Self::Current),
            "leasing" => Some(Self::Leasing),
            "savings_mode" => Some(Self::SavingsMode),
            "house_purchase" => Some(Self::HousePurchase),
            "custom" => Some(Self::Custom),
            "allocation_target" => Some(Self::AllocationTarget),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentDirection {
    AddOutflow,
    RemoveOutflow,
    AddInflow,
    RemoveInflow,
}

impl AdjustmentDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AddOutflow => "add_outflow",
            Self::RemoveOutflow => "remove_outflow",
            Self::AddInflow => "add_inflow",
            Self::RemoveInflow => "remove_inflow",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "add_outflow" => Some(Self::AddOutflow),
            "remove_outflow" => Some(Self::RemoveOutflow),
            "add_inflow" => Some(Self::AddInflow),
            "remove_inflow" => Some(Self::RemoveInflow),
            _ => None,
        }
    }

    pub fn signed_multiplier(&self) -> f64 {
        match self {
            Self::AddOutflow | Self::RemoveInflow => -1.0,
            Self::RemoveOutflow | Self::AddInflow => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentFrequency {
    Monthly,
    Weekly,
    Quarterly,
    OneTime,
}

impl AdjustmentFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Weekly => "weekly",
            Self::Quarterly => "quarterly",
            Self::OneTime => "one_time",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "monthly" => Some(Self::Monthly),
            "weekly" => Some(Self::Weekly),
            "quarterly" => Some(Self::Quarterly),
            "one_time" => Some(Self::OneTime),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentTarget {
    Household,
    Subscription,
    Category,
    CustomLabel,
    AllocationTarget,
}

impl AdjustmentTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Household => "household",
            Self::Subscription => "subscription",
            Self::Category => "category",
            Self::CustomLabel => "custom_label",
            Self::AllocationTarget => "allocation_target",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "household" => Some(Self::Household),
            "subscription" => Some(Self::Subscription),
            "category" => Some(Self::Category),
            "custom_label" => Some(Self::CustomLabel),
            "allocation_target" => Some(Self::AllocationTarget),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlanAdjustment {
    pub id: Uuid,
    pub version_id: Uuid,
    pub direction: AdjustmentDirection,
    pub amount: f64,
    pub frequency: AdjustmentFrequency,
    pub target_type: AdjustmentTarget,
    pub target_key: Option<String>,
    pub label: Option<String>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct ConfirmedSubscription {
    pub payee_key: String,
    pub amount: f64,
    pub interval_days: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlanRow {
    pub id: Uuid,
    pub name: String,
    pub template: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VersionRow {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub version_number: i32,
    pub is_latest: bool,
    pub frozen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub baseline_computation_id: Option<Uuid>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdjustmentRow {
    pub id: Uuid,
    pub version_id: Uuid,
    pub direction: String,
    pub amount: f64,
    pub frequency: String,
    pub target_type: String,
    pub target_key: Option<String>,
    pub label: Option<String>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub sort_order: i32,
}

impl AdjustmentRow {
    pub fn into_adjustment(self) -> PlanAdjustment {
        PlanAdjustment {
            id: self.id,
            version_id: self.version_id,
            direction: AdjustmentDirection::from_str(&self.direction).unwrap_or(AdjustmentDirection::AddOutflow),
            amount: self.amount,
            frequency: AdjustmentFrequency::from_str(&self.frequency).unwrap_or(AdjustmentFrequency::Monthly),
            target_type: AdjustmentTarget::from_str(&self.target_type).unwrap_or(AdjustmentTarget::Household),
            target_key: self.target_key,
            label: self.label,
            effective_from: self.effective_from,
            effective_to: self.effective_to,
            sort_order: self.sort_order,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DailyNetPoint {
    pub date: NaiveDate,
    pub planned_net: f64,
    pub planned_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanListItem {
    pub id: String,
    pub name: String,
    pub template: String,
    pub is_active: bool,
    pub latest_version_id: Option<String>,
    pub latest_version_number: Option<i32>,
    pub plan_stale: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivePlanInfo {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub latest_version_id: Uuid,
    pub latest_version_number: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareVersionMetrics {
    pub version_id: String,
    pub version_number: i32,
    pub frozen: bool,
    pub monthly_delta_sum: String,
    pub projected_month_end_balance: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareResponse {
    pub plan_id: String,
    pub plan_name: String,
    pub versions: Vec<CompareVersionMetrics>,
    pub at_version_cap: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanVsActualRow {
    pub date: String,
    pub planned: Option<String>,
    pub actual: Option<String>,
    pub deviation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanVsActualResponse {
    pub month: String,
    pub reporting_currency: String,
    pub plan_stale: bool,
    pub actuals_stale: bool,
    pub rows: Vec<PlanVsActualRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavingsSuggestion {
    pub pattern_id: String,
    pub payee_key: String,
    pub display_name: String,
    pub current_amount: String,
    pub interval_days: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanProjection {
    pub source: String,
    pub monthly_delta_sum: String,
    pub projected_month_end_balance: String,
    pub reporting_currency: String,
    pub key_metrics: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EphemeralPlanDraft {
    pub template: Option<String>,
    pub adjustments: Vec<PlanAdjustment>,
}

pub fn fmt_amount(value: f64) -> String {
    format!("{:.2}", value)
}
