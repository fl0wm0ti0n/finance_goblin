pub mod classify;
pub mod detection;
pub mod discovery;
pub mod price_change;
pub mod repository;
pub mod service;
pub mod tags;
pub mod types;

pub use service::SubscriptionService;
pub use types::{ConfirmedRecurring, DetectionResult};
