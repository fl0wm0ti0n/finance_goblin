use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Pending,
    Confirmed,
    Rejected,
    Inactive,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::Inactive => "inactive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "confirmed" => Some(Self::Confirmed),
            "rejected" => Some(Self::Rejected),
            "inactive" => Some(Self::Inactive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionKind {
    Subscription,
    StandingOrder,
}

impl SubscriptionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Subscription => "subscription",
            Self::StandingOrder => "standing_order",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "subscription" => Some(Self::Subscription),
            "standing_order" => Some(Self::StandingOrder),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    NewDetection,
    PriceChange,
    IntervalChange,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewDetection => "new_detection",
            Self::PriceChange => "price_change",
            Self::IntervalChange => "interval_change",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfirmedRecurring {
    pub payee_key: String,
    pub amount: f64,
    pub interval_days: i64,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Default)]
pub struct DetectionResult {
    pub confirmed_recurring: Vec<ConfirmedRecurring>,
    /// All rejected fingerprints (subscription detection pipeline).
    pub rejected_fingerprints: HashSet<String>,
    /// Subscription-kind rejections only — forecast still projects rejected standing orders.
    pub forecast_excluded_rejections: HashSet<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PatternRow {
    pub id: Uuid,
    pub fingerprint: String,
    pub status: String,
    pub kind: String,
    pub payee_key: String,
    pub display_name: String,
    pub interval_days: i32,
    pub current_amount: f64,
    pub confidence_pct: i16,
    pub first_seen_at: chrono::NaiveDate,
    pub last_seen_at: chrono::NaiveDate,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PatternRow {
    pub fn amount_f64(&self) -> f64 {
        self.current_amount
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PatternDetailRow {
    pub id: Uuid,
    pub fingerprint: String,
    pub status: String,
    pub kind: String,
    pub payee_key: String,
    pub display_name: String,
    pub interval_days: i32,
    pub current_amount: f64,
    pub confidence_pct: i16,
    pub first_seen_at: chrono::NaiveDate,
    pub last_seen_at: chrono::NaiveDate,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PriceEventRow {
    pub id: Uuid,
    pub event_type: String,
    pub amount: f64,
    pub previous_amount: Option<f64>,
    pub delta_pct: Option<f64>,
    pub interval_days: Option<i32>,
    pub occurred_at: chrono::NaiveDate,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct AlertRow {
    pub id: Uuid,
    pub pattern_id: Option<Uuid>,
    pub alert_type: String,
    pub title: String,
    pub body: Option<String>,
    pub read_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
