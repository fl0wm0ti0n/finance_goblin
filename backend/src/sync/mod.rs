use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::alerts::{AlertService, EvalContext};
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::exchanges::ExchangeService;
use crate::firefly::{FireflyClient, FireflyError};
use crate::forecast::ForecastService;
use crate::forecast_ml::service::ForecastMlError;
use crate::forecast_ml::ForecastMlService;
use crate::plan::risk::PlanRiskService;
use crate::portfolio::PortfolioEngine;
use crate::subscriptions::SubscriptionService;
use crate::wealth::WealthService;

#[derive(Clone)]
pub struct SyncService {
    config: AppConfig,
    db: DbPool,
    forecast: ForecastService,
    forecast_ml: ForecastMlService,
    plan_risk: PlanRiskService,
    subscriptions: SubscriptionService,
    alerts: AlertService,
    wealth: WealthService,
    exchanges: ExchangeService,
    portfolio: PortfolioEngine,
    active_run: Arc<Mutex<Option<Uuid>>>,
    phase: Arc<Mutex<Option<String>>>,
    last_firefly_sync: Arc<Mutex<Option<DateTime<Utc>>>>,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub state: String,
    pub phase: Option<String>,
    pub active_run_id: Option<Uuid>,
    pub last_run: Option<SyncRunRow>,
    pub last_firefly_run: Option<SyncRunRow>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SyncRunRow {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
    pub trigger: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EntityCounts {
    pub entities: Vec<EntityCount>,
}

#[derive(Debug, Serialize)]
pub struct EntityCount {
    pub entity: String,
    pub count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum TriggerError {
    #[error("sync already running")]
    AlreadyRunning { run_id: Uuid },
}

impl SyncService {
    pub fn new(
        config: AppConfig,
        db: DbPool,
        forecast: ForecastService,
        forecast_ml: ForecastMlService,
        plan_risk: PlanRiskService,
        subscriptions: SubscriptionService,
        alerts: AlertService,
        wealth: WealthService,
        exchanges: ExchangeService,
        portfolio: PortfolioEngine,
    ) -> Self {
        Self {
            config,
            db,
            forecast,
            forecast_ml,
            plan_risk,
            subscriptions,
            alerts,
            wealth,
            exchanges,
            portfolio,
            active_run: Arc::new(Mutex::new(None)),
            phase: Arc::new(Mutex::new(None)),
            last_firefly_sync: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_scheduler(&self) -> anyhow::Result<()> {
        let firefly_interval = self.config.sync.interval_seconds;
        let exchange_interval = self.config.exchanges.interval_seconds;

        if firefly_interval == 0 && exchange_interval == 0 {
            info!("sync scheduler disabled (interval_seconds=0)");
            return Ok(());
        }

        let service = self.clone();
        tokio::spawn(async move {
            let tick_secs = firefly_interval.max(exchange_interval).max(60);
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(tick_secs));
            ticker.tick().await;
            loop {
                ticker.tick().await;
                if service.active_run.lock().await.is_some() {
                    continue;
                }

                let last_ff = *service.last_firefly_sync.lock().await;
                let ff_due = firefly_interval > 0
                    && last_ff
                        .map(|t| {
                            (Utc::now() - t).num_seconds() as u64 >= firefly_interval
                        })
                        .unwrap_or(true);

                if ff_due {
                    match service.trigger("scheduled").await {
                        Ok(id) => info!(%id, "scheduled full sync triggered"),
                        Err(TriggerError::AlreadyRunning { .. }) => {}
                    }
                } else if exchange_interval > 0 && service.exchanges.enabled() {
                    match service.trigger_exchanges_only("scheduled_exchanges").await {
                        Ok(id) => info!(%id, "scheduled exchange-only sync triggered"),
                        Err(TriggerError::AlreadyRunning { .. }) => {}
                    }
                }
            }
        });

        info!(
            firefly_interval_seconds = firefly_interval,
            exchange_interval_seconds = exchange_interval,
            "sync scheduler started"
        );
        Ok(())
    }

    pub async fn trigger_manual(&self) -> Result<Uuid, TriggerError> {
        self.trigger("manual").await
    }

    pub async fn trigger_exchanges_manual(&self) -> Result<Uuid, TriggerError> {
        self.trigger_exchanges_only("manual_exchanges").await
    }

    async fn trigger(&self, trigger: &str) -> Result<Uuid, TriggerError> {
        self.spawn_run(trigger, RunMode::Full).await
    }

    async fn trigger_exchanges_only(&self, trigger: &str) -> Result<Uuid, TriggerError> {
        self.spawn_run(trigger, RunMode::ExchangesOnly).await
    }

    async fn spawn_run(&self, trigger: &str, mode: RunMode) -> Result<Uuid, TriggerError> {
        let mut guard = self.active_run.lock().await;
        if let Some(run_id) = *guard {
            return Err(TriggerError::AlreadyRunning { run_id });
        }

        let run_id = Uuid::new_v4();
        crate::db::repositories::insert_sync_run(self.db.pool(), run_id, trigger)
            .await
            .map_err(|_| TriggerError::AlreadyRunning {
                run_id: Uuid::nil(),
            })?;
        *guard = Some(run_id);
        drop(guard);

        let service = self.clone();
        let trigger_label = trigger.to_string();
        tokio::spawn(async move {
            if let Err(e) = service.execute_run(run_id, &trigger_label, mode).await {
                error!(?e, %run_id, "sync run failed");
            }
        });

        Ok(run_id)
    }

    async fn execute_run(
        &self,
        run_id: Uuid,
        trigger: &str,
        mode: RunMode,
    ) -> Result<(), FireflyError> {
        info!(%run_id, ?mode, "sync run started");

        let mut forecast_id = None;

        if mode == RunMode::Full {
            *self.phase.lock().await = Some("sync".into());
            if !self.config.firefly.pat_configured() {
                let msg = "firefly_personal_access_token_missing: set non-empty FIREFLY_PERSONAL_ACCESS_TOKEN (see docs/engineering/runbook.md § Omniflow external deploy)";
                crate::db::repositories::finish_sync_run(
                    self.db.pool(),
                    run_id,
                    "failed",
                    Some(msg),
                )
                .await?;
                *self.phase.lock().await = None;
                *self.active_run.lock().await = None;
                return Err(FireflyError::PersonalAccessTokenMissing);
            }
            let client = FireflyClient::new(&self.config.firefly, self.db.pool().clone());

            let result = async {
                crate::firefly::sync::sync_reference_entities(&client, self.db.pool()).await?;
                crate::firefly::sync::sync_transactions(
                    &client,
                    self.db.pool(),
                    self.config.sync.overlap_days,
                    trigger,
                )
                .await?;
                Ok::<(), FireflyError>(())
            }
            .await;

            match &result {
                Ok(()) => {
                    crate::db::repositories::finish_sync_run(
                        self.db.pool(),
                        run_id,
                        "success",
                        None,
                    )
                    .await?;
                    *self.last_firefly_sync.lock().await = Some(Utc::now());
                    info!(%run_id, "firefly sync succeeded");
                }
                Err(e) => {
                    crate::db::repositories::finish_sync_run(
                        self.db.pool(),
                        run_id,
                        "failed",
                        Some(&e.to_string()),
                    )
                    .await?;
                    *self.phase.lock().await = None;
                    *self.active_run.lock().await = None;
                    return result;
                }
            }

            *self.phase.lock().await = Some("subscriptions".into());
            let detection_started = std::time::Instant::now();
            let detection_result = match self.subscriptions.run_detection(run_id).await {
                Ok(result) => Some(result),
                Err(e) => {
                    warn!(?e, %run_id, "subscription detection failed; using prior confirmed state");
                    Some(self.subscriptions.fallback_detection_result().await)
                }
            };
            info!(
                %run_id,
                duration_ms = detection_started.elapsed().as_millis(),
                "subscription detection phase complete"
            );

            *self.phase.lock().await = Some("forecast".into());
            let forecast_ctx = detection_result.as_ref();
            forecast_id = match self.forecast.recompute(run_id, forecast_ctx).await {
                Ok(id) => Some(id),
                Err(e) => {
                    warn!(?e, %run_id, "forecast recompute failed; serving stale snapshot");
                    self.forecast
                        .repository()
                        .latest_successful()
                        .await
                        .ok()
                        .flatten()
                        .map(|r| r.id)
                }
            };

            if let Some(baseline_id) = forecast_id {
                if self.config.forecast_ml.enabled {
                    *self.phase.lock().await = Some("forecast_ml".into());
                    let ml_started = std::time::Instant::now();
                    match self.forecast_ml.recompute(run_id, baseline_id).await {
                        Ok(Some(_ml_id)) => {
                            info!(
                                %run_id,
                                duration_ms = ml_started.elapsed().as_millis(),
                                "forecast_ml phase complete"
                            );
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!(?e, %run_id, "ML forecast skipped; baseline unaffected");
                            let _ = self
                                .forecast_ml
                                .record_skip_on_baseline(baseline_id, &e)
                                .await;
                        }
                    }
                    self.trigger_plan_risk_refresh().await;
                } else {
                    let _ = self
                        .forecast_ml
                        .record_skip_on_baseline(baseline_id, &ForecastMlError::Disabled)
                        .await;
                }
            }
        } else {
            forecast_id = self
                .forecast
                .repository()
                .latest_successful()
                .await
                .ok()
                .flatten()
                .map(|r| r.id);
        }

        let exchanges_result = self.run_exchanges_and_alerts(run_id, forecast_id).await;

        if mode == RunMode::ExchangesOnly {
            let (status, error_message) = exchanges_only_terminal(&exchanges_result);
            crate::db::repositories::finish_sync_run(
                self.db.pool(),
                run_id,
                status,
                error_message.as_deref(),
            )
            .await?;
            if exchanges_result.is_err() {
                *self.phase.lock().await = None;
                *self.active_run.lock().await = None;
                return exchanges_result;
            }
        }

        *self.phase.lock().await = None;
        *self.active_run.lock().await = None;
        exchanges_result
    }

    async fn run_exchanges_and_alerts(
        &self,
        run_id: Uuid,
        forecast_id: Option<Uuid>,
    ) -> Result<(), FireflyError> {
        *self.phase.lock().await = Some("exchanges".into());
        let exchanges_started = std::time::Instant::now();

        if self.exchanges.enabled() {
            if let Err(e) = self.exchanges.run_post_sync(run_id).await {
                warn!(?e, %run_id, "exchange sync phase error");
            }
        }

        info!(
            %run_id,
            duration_ms = exchanges_started.elapsed().as_millis(),
            "exchanges phase complete"
        );

        if let Some(latest_forecast_id) = forecast_id {
            *self.phase.lock().await = Some("alerts".into());
            let alerts_started = std::time::Instant::now();

            let exchange_repo = self.exchanges.repository();
            if let Err(e) = self
                .wealth
                .upsert_daily_snapshot(
                    run_id,
                    Some(exchange_repo.as_ref()),
                    Some(&self.portfolio),
                )
                .await
            {
                warn!(?e, %run_id, "wealth snapshot upsert failed");
            }

            let plan_computation_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT pc.id FROM plan_computations pc
                JOIN plan_versions v ON v.id = pc.version_id
                JOIN plans p ON p.id = v.plan_id
                WHERE p.is_active = true AND v.is_latest = true
                  AND pc.status = 'success'
                ORDER BY pc.computed_at DESC LIMIT 1
                "#,
            )
            .fetch_optional(self.db.pool())
            .await
            .ok()
            .flatten();

            let eval_context = EvalContext {
                forecast_computation_id: latest_forecast_id,
                plan_computation_id,
                config: self.alerts.config().clone(),
            };

            if let Err(e) = self.alerts.run_post_sync(run_id, eval_context).await {
                warn!(?e, %run_id, "alert evaluation failed");
            }

            info!(
                %run_id,
                duration_ms = alerts_started.elapsed().as_millis(),
                "alerts phase complete"
            );
        }

        Ok(())
    }

    async fn trigger_plan_risk_refresh(&self) {
        let plan_computation_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT pc.id FROM plan_computations pc
            JOIN plan_versions v ON v.id = pc.version_id
            JOIN plans p ON p.id = v.plan_id
            WHERE p.is_active = true AND v.is_latest = true
              AND pc.status = 'success'
            ORDER BY pc.computed_at DESC LIMIT 1
            "#,
        )
        .fetch_optional(self.db.pool())
        .await
        .ok()
        .flatten();

        if let Some(pc_id) = plan_computation_id {
            if let Err(e) = self.plan_risk.compute(pc_id).await {
                warn!(?e, "plan risk score refresh failed");
            }
        }
    }

    pub fn phase_label(phase: &str) -> &str {
        match phase {
            "forecast_ml" => "ML forecast…",
            _ => phase,
        }
    }

    pub async fn status(&self) -> SyncStatusResponse {
        let active_run_id = *self.active_run.lock().await;
        let phase = self.phase.lock().await.clone();
        let last_run = self.latest_run().await.ok().flatten();
        let last_firefly_run = self.latest_firefly_run().await.ok().flatten();

        let state = if active_run_id.is_some() {
            "running".to_string()
        } else {
            last_run
                .as_ref()
                .map(|r| r.status.clone())
                .unwrap_or_else(|| "idle".to_string())
        };

        SyncStatusResponse {
            state,
            phase,
            active_run_id,
            last_run,
            last_firefly_run,
        }
    }

    pub async fn list_runs(&self, limit: i64) -> Result<Vec<SyncRunRow>, sqlx::Error> {
        sqlx::query_as::<_, SyncRunRow>(
            r#"
            SELECT id, started_at, finished_at, status, trigger, error_message
            FROM sync_runs ORDER BY started_at DESC LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
    }

    pub async fn entity_counts(&self) -> Result<EntityCounts, sqlx::Error> {
        let tables = [
            "accounts",
            "transactions",
            "categories",
            "budgets",
            "tags",
            "piggy_banks",
        ];
        let mut entities = Vec::new();
        for table in tables {
            let count = self.db.entity_count(table).await?;
            entities.push(EntityCount {
                entity: table.to_string(),
                count,
            });
        }

        if self.exchanges.enabled() {
            let connections = self.exchanges.list_connections().await?;
            for conn in connections {
                entities.push(EntityCount {
                    entity: format!("exchange_{}", conn.id),
                    count: conn.counts.holdings,
                });
            }
        }

        Ok(EntityCounts { entities })
    }

    pub async fn exchange_status(&self) -> Result<Vec<crate::exchanges::types::ExchangeListItem>, sqlx::Error> {
        self.exchanges.list_connections().await
    }

    async fn latest_run(&self) -> Result<Option<SyncRunRow>, sqlx::Error> {
        sqlx::query_as::<_, SyncRunRow>(
            r#"
            SELECT id, started_at, finished_at, status, trigger, error_message
            FROM sync_runs ORDER BY started_at DESC LIMIT 1
            "#,
        )
        .fetch_optional(self.db.pool())
        .await
    }

    async fn latest_firefly_run(&self) -> Result<Option<SyncRunRow>, sqlx::Error> {
        sqlx::query_as::<_, SyncRunRow>(
            r#"
            SELECT id, started_at, finished_at, status, trigger, error_message
            FROM sync_runs
            WHERE trigger IN ('manual', 'scheduled')
            ORDER BY started_at DESC LIMIT 1
            "#,
        )
        .fetch_optional(self.db.pool())
        .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Full,
    ExchangesOnly,
}

fn exchanges_only_terminal(result: &Result<(), FireflyError>) -> (&'static str, Option<String>) {
    match result {
        Ok(()) => ("success", None),
        Err(e) => ("failed", Some(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{exchanges_only_terminal, RunMode};
    use crate::firefly::FireflyError;

    #[test]
    fn exchanges_phase_after_forecast_in_full_run() {
        assert_eq!(RunMode::Full, RunMode::Full);
        assert_ne!(RunMode::Full, RunMode::ExchangesOnly);
    }

    #[test]
    fn exchanges_only_finishes_success_on_ok() {
        let result: Result<(), FireflyError> = Ok(());
        let (status, msg) = exchanges_only_terminal(&result);
        assert_eq!(status, "success");
        assert!(msg.is_none());
    }

    #[test]
    fn exchanges_only_finishes_failed_on_err() {
        let result: Result<(), FireflyError> = Err(FireflyError::PersonalAccessTokenMissing);
        let (status, msg) = exchanges_only_terminal(&result);
        assert_eq!(status, "failed");
        assert!(
            msg.as_deref()
                .unwrap_or("")
                .starts_with("firefly_personal_access_token_missing")
        );
    }
}
