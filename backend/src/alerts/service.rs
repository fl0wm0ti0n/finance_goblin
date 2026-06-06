use std::collections::HashSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::config::AlertsConfig;
use crate::db::DbPool;
use crate::plan::PlanService;
use crate::wealth::WealthService;

use super::evaluate::{evaluate_budget_drift, evaluate_plan_viability, evaluate_scarcity, EvaluateContext};
use super::repository::AlertRepository;
use super::types::{
    AlertCandidate, AlertError, AlertEvalResult, AlertListFilter, AlertRow, EvalContext,
};

#[derive(Clone)]
pub struct AlertService {
    repo: Arc<AlertRepository>,
    wealth: WealthService,
    plans: Option<PlanService>,
    config: AlertsConfig,
}

impl AlertService {
    pub fn new(
        db: DbPool,
        config: AlertsConfig,
        wealth: WealthService,
        plans: Option<PlanService>,
    ) -> Self {
        let repo = AlertRepository::new(db.pool().clone());
        Self {
            repo: Arc::new(repo),
            wealth,
            plans,
            config,
        }
    }

    pub fn config(&self) -> &AlertsConfig {
        &self.config
    }

    pub fn repository(&self) -> &AlertRepository {
        &self.repo
    }

    pub async fn mirror_config_at_startup(&self) -> Result<(), AlertError> {
        self.repo.mirror_config(&self.config).await?;
        Ok(())
    }

    pub async fn run_post_sync(
        &self,
        run_id: Uuid,
        ctx: EvalContext,
    ) -> Result<AlertEvalResult, AlertError> {
        let mut created = 0u32;
        let mut updated = 0u32;
        let mut resolved = 0u32;

        let eval_ctx = EvaluateContext {
            pool: self.repo.pool(),
            forecast_computation_id: ctx.forecast_computation_id,
            plan_computation_id: ctx.plan_computation_id,
            config: &ctx.config,
        };

        let mut candidates: Vec<AlertCandidate> = Vec::new();

        candidates.extend(evaluate_scarcity(&eval_ctx).await?);

        if let Some(ref plans) = self.plans {
            if let Some(active) = plans.active_plan().await.map_err(|e| {
                AlertError::Db(sqlx::Error::Protocol(format!("plan error: {e}")))
            })? {
                let adjustments = plans
                    .load_adjustments(active.latest_version_id)
                    .await
                    .map_err(|e| AlertError::Db(sqlx::Error::Protocol(format!("plan error: {e}"))))?;
                candidates.extend(evaluate_budget_drift(&eval_ctx, &adjustments).await?);

                if let Some(plan_comp_id) = ctx.plan_computation_id {
                    candidates.extend(
                        evaluate_plan_viability(
                            &eval_ctx,
                            active.plan_id,
                            active.latest_version_id,
                            plan_comp_id,
                        )
                        .await?,
                    );
                }
            }
        }

        let candidate_fps: HashSet<String> =
            candidates.iter().map(|c| c.fingerprint.clone()).collect();

        for candidate in &candidates {
            if self.repo.upsert_or_resolve(candidate, run_id).await? {
                created += 1;
            } else {
                updated += 1;
            }
        }

        let active_fps = self.repo.active_fingerprints().await?;
        for fp in active_fps {
            if !candidate_fps.contains(&fp) {
                if self.repo.resolve_by_fingerprint(&fp).await? {
                    resolved += 1;
                }
            }
        }

        Ok(AlertEvalResult {
            created,
            updated,
            resolved,
        })
    }

    pub async fn budget_status_for_tool(
        &self,
        category_filter: Option<String>,
    ) -> Result<serde_json::Value, AlertError> {
        let Some(ref plans) = self.plans else {
            return Ok(serde_json::json!({ "error": "no_active_plan" }));
        };
        let active = plans
            .active_plan()
            .await
            .map_err(|e| AlertError::Db(sqlx::Error::Protocol(format!("{e}"))))?;
        let Some(active) = active else {
            return Ok(serde_json::json!({ "error": "no_active_plan" }));
        };

        let Some(forecast_id) = self.repo.latest_forecast_computation_id().await? else {
            return Ok(serde_json::json!({ "error": "no_forecast" }));
        };

        let adjustments = plans
            .load_adjustments(active.latest_version_id)
            .await
            .map_err(|e| AlertError::Db(sqlx::Error::Protocol(format!("{e}"))))?;

        let eval_ctx = EvaluateContext {
            pool: self.repo.pool(),
            forecast_computation_id: forecast_id,
            plan_computation_id: None,
            config: &self.config,
        };

        let mut candidates = evaluate_budget_drift(&eval_ctx, &adjustments).await?;
        if let Some(cat) = category_filter {
            candidates.retain(|c| c.entity_id.as_deref() == Some(cat.as_str()));
        }

        let categories: Vec<serde_json::Value> = candidates
            .iter()
            .map(|c| {
                serde_json::json!({
                    "category_id": c.entity_id,
                    "title": c.title,
                    "message": c.message,
                    "context": c.context,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "month": format!("{}", chrono::Utc::now().format("%Y-%m")),
            "budget_drift_pct": self.config.budget_drift_pct,
            "categories": categories,
        }))
    }

    pub async fn list(&self, filter: AlertListFilter) -> Result<Vec<AlertRow>, AlertError> {
        let latest_forecast: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM forecast_computations WHERE status = 'success' ORDER BY computed_at DESC LIMIT 1",
        )
        .fetch_optional(self.repo.pool())
        .await?;

        let mut rows = self.repo.list(&filter).await?;
        for row in &mut rows {
            row.stale = is_stale(&row.context, latest_forecast);
        }
        Ok(rows)
    }

    pub async fn unread_count(&self) -> Result<u32, AlertError> {
        Ok(self.repo.unread_count().await?)
    }

    pub async fn acknowledge(&self, id: Uuid) -> Result<AlertRow, AlertError> {
        self.repo
            .acknowledge(id)
            .await?
            .ok_or(AlertError::NotFound)
    }

    pub async fn dismiss(&self, id: Uuid) -> Result<AlertRow, AlertError> {
        self.repo.dismiss(id).await?.ok_or(AlertError::NotFound)
    }
}

fn is_stale(context: &serde_json::Value, latest_forecast: Option<Uuid>) -> bool {
    if let (Some(latest), Some(bound)) = (
        latest_forecast,
        context
            .get("forecast_computation_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
    ) {
        return bound != latest;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_when_computation_id_differs() {
        let latest = Uuid::new_v4();
        let bound = Uuid::new_v4();
        let ctx = serde_json::json!({ "forecast_computation_id": bound.to_string() });
        assert!(is_stale(&ctx, Some(latest)));
    }
}
