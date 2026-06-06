use std::sync::Arc;

use tracing::{info, warn};
use uuid::Uuid;

use crate::config::SubscriptionsConfig;
use crate::db::DbPool;

use super::detection::DetectionPipeline;
use super::repository::SubscriptionRepository;
use super::types::DetectionResult;

#[derive(Clone)]
pub struct SubscriptionService {
    repo: Arc<SubscriptionRepository>,
}

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}

impl SubscriptionService {
    pub fn new(db: DbPool, config: SubscriptionsConfig) -> Self {
        let repo = SubscriptionRepository::new(db.pool().clone(), config);
        Self {
            repo: Arc::new(repo),
        }
    }

    pub fn repository(&self) -> &SubscriptionRepository {
        &self.repo
    }

    pub async fn run_detection(&self, sync_run_id: Uuid) -> Result<DetectionResult, SubscriptionError> {
        let started = std::time::Instant::now();
        let rejections = self.repo.load_rejection_fingerprints().await?;
        let forecast_excluded = self.repo.load_forecast_excluded_rejections().await?;
        let confirmed_fps = self.repo.load_confirmed_fingerprints().await?;

        let pipeline = DetectionPipeline::new(&self.repo);
        let _candidates = pipeline
            .run_candidates(sync_run_id, &rejections, &confirmed_fps)
            .await?;

        pipeline.process_confirmed(sync_run_id).await?;

        let confirmed = self.repo.load_confirmed_for_forecast().await?;
        info!(
            %sync_run_id,
            confirmed_count = confirmed.len(),
            duration_ms = started.elapsed().as_millis(),
            "subscription detection complete"
        );

        Ok(DetectionResult {
            confirmed_recurring: confirmed,
            rejected_fingerprints: rejections,
            forecast_excluded_rejections: forecast_excluded,
        })
    }

    pub async fn list_patterns(
        &self,
        status: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, SubscriptionError> {
        let rows = self.repo.list_patterns(status, kind).await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id.to_string(),
                    "display_name": r.display_name,
                    "status": r.status,
                    "kind": r.kind,
                    "current_amount": r.amount_f64(),
                    "confidence_pct": r.confidence_pct,
                    "interval_days": r.interval_days,
                })
            })
            .collect())
    }

    pub async fn recent_price_events(&self, limit: i64) -> Result<Vec<serde_json::Value>, SubscriptionError> {
        let rows = self.repo.recent_price_events(limit).await?;
        Ok(rows
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "amount": e.amount,
                    "event_type": e.event_type,
                    "occurred_at": e.occurred_at.to_string(),
                    "delta_pct": e.delta_pct,
                })
            })
            .collect())
    }

    pub async fn fallback_detection_result(&self) -> DetectionResult {
        match (
            self.repo.load_confirmed_for_forecast().await,
            self.repo.load_rejection_fingerprints().await,
            self.repo.load_forecast_excluded_rejections().await,
        ) {
            (Ok(confirmed), Ok(rejected), Ok(forecast_excluded)) => DetectionResult {
                confirmed_recurring: confirmed,
                rejected_fingerprints: rejected,
                forecast_excluded_rejections: forecast_excluded,
            },
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                warn!(?e, "failed to load fallback subscription context");
                DetectionResult::default()
            }
        }
    }
}
