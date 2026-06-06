use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use super::project::{project_account, HouseholdIncomeContext};
use super::recurring::detect_inflow_patterns;
use super::repository::{balance_warning_entry, build_metadata, ForecastRepository};
use crate::config::ForecastConfig;
use crate::db::DbPool;
use crate::plan::PlanService;
use crate::subscriptions::DetectionResult;

#[derive(Clone)]
pub struct ForecastService {
    repo: Arc<ForecastRepository>,
    plan_service: Arc<RwLock<Option<PlanService>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ForecastError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}

impl ForecastService {
    pub fn new(db: DbPool, config: ForecastConfig) -> Self {
        let repo = ForecastRepository::new(db.pool().clone(), config);
        Self {
            repo: Arc::new(repo),
            plan_service: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn attach_plan_service(&self, plans: PlanService) {
        *self.plan_service.write().await = Some(plans);
    }

    pub fn repository(&self) -> &ForecastRepository {
        &self.repo
    }

    pub fn repository_arc(&self) -> Arc<ForecastRepository> {
        Arc::clone(&self.repo)
    }

    pub async fn recompute(
        &self,
        sync_run_id: Uuid,
        subscription_context: Option<&DetectionResult>,
    ) -> Result<Uuid, ForecastError> {
        let computation_id = Uuid::new_v4();
        self.repo
            .insert_computation(computation_id, sync_run_id)
            .await?;

        let result = self.run_projection(computation_id, subscription_context).await;
        match result {
            Ok(metadata) => {
                self.repo.mark_success(computation_id, &metadata).await?;
                self.repo.enforce_retention().await?;
                info!(%computation_id, "forecast recompute succeeded");

                if let Some(plans) = self.plan_service.read().await.clone() {
                    let plan_started = std::time::Instant::now();
                    if let Err(e) = plans.refresh_active_after_forecast(computation_id).await {
                        warn!(?e, %computation_id, "active plan post-forecast refresh error");
                    }
                    info!(
                        %computation_id,
                        duration_ms = plan_started.elapsed().as_millis(),
                        "plan post-forecast hook complete"
                    );
                }

                Ok(computation_id)
            }
            Err(e) => {
                warn!(?e, %computation_id, "forecast recompute failed");
                self.repo
                    .mark_failed(computation_id, &e.to_string())
                    .await?;
                Err(e)
            }
        }
    }

    async fn run_projection(
        &self,
        computation_id: Uuid,
        subscription_context: Option<&DetectionResult>,
    ) -> Result<serde_json::Value, ForecastError> {
        let accounts = self.repo.list_asset_accounts().await?;
        let category_names = self.repo.category_name_map().await?;
        let config = self.repo.config().clone();

        let mut revenue_reference_txs = Vec::new();
        for rev in self.repo.list_revenue_accounts().await? {
            let txs = self.repo.fetch_transactions_for_account(&rev.firefly_id).await?;
            revenue_reference_txs.extend(txs);
        }
        let household_income_patterns =
            detect_inflow_patterns(&revenue_reference_txs, config.recurring_amount_tolerance_pct);
        let household_income = if household_income_patterns.is_empty() {
            None
        } else {
            Some(HouseholdIncomeContext {
                patterns: &household_income_patterns,
                reference_transactions: &revenue_reference_txs,
            })
        };

        let mut account_flags: HashMap<String, bool> = HashMap::new();
        let mut balance_warnings: Vec<serde_json::Value> = Vec::new();

        for account in &accounts {
            let account_id = &account.firefly_id;
            let started = std::time::Instant::now();

            let txs = self.repo.fetch_transactions_for_account(account_id).await?;
            let starting_balance = account.balance.unwrap_or(0.0);

            if let Some(warning) =
                balance_warning_entry(account_id, starting_balance, txs.len())
            {
                balance_warnings.push(warning);
            }

            let projection = project_account(
                starting_balance,
                &txs,
                &category_names,
                &config,
                subscription_context,
                household_income.as_ref(),
            );

            account_flags.insert(account_id.clone(), projection.low_confidence);

            self.repo
                .bulk_insert_daily(computation_id, account_id, &projection.daily)
                .await?;
            self.repo
                .bulk_insert_monthly(computation_id, account_id, &projection.monthly)
                .await?;

            info!(
                %account_id,
                duration_ms = started.elapsed().as_millis(),
                days = projection.daily.len(),
                "forecast account projection complete"
            );
        }

        Ok(build_metadata(&account_flags, &balance_warnings))
    }

    pub async fn aggregate_daily_balances(
        &self,
        computation_id: Uuid,
        currency: Option<&str>,
    ) -> Result<Vec<super::types::DailyPoint>, ForecastError> {
        let accounts = self.repo.list_asset_accounts().await?;
        let filtered: Vec<_> = accounts
            .into_iter()
            .filter(|a| {
                currency.map(|c| a.currency.as_deref() == Some(c)).unwrap_or(true)
            })
            .collect();

        let mut by_date: HashMap<chrono::NaiveDate, f64> = HashMap::new();
        for account in filtered {
            let series = self
                .repo
                .fetch_daily_series(computation_id, &account.firefly_id, None, None)
                .await?;
            for point in series {
                *by_date.entry(point.date).or_insert(0.0) += point.balance;
            }
        }

        let mut points: Vec<_> = by_date
            .into_iter()
            .map(|(date, balance)| super::types::DailyPoint { date, balance })
            .collect();
        points.sort_by_key(|p| p.date);
        Ok(points)
    }

    pub async fn forecast_summary_for_tool(
        &self,
        horizon: String,
        account_id: Option<String>,
        variant: Option<String>,
    ) -> Result<serde_json::Value, ForecastError> {
        let model_kind = match variant.as_deref().unwrap_or("baseline") {
            "ml_enhanced" => "ml_enhanced",
            _ => "baseline",
        };

        let latest = self
            .repo
            .latest_successful_by_kind(model_kind)
            .await?
            .ok_or_else(|| ForecastError::Db(sqlx::Error::RowNotFound))?;

        let days = match horizon.as_str() {
            "24m" => 730,
            "12m" => 365,
            "6m" => 180,
            _ => 90,
        };
        let today = chrono::Utc::now().date_naive();
        let end = today + chrono::Duration::days(days);

        let payload = if model_kind == "ml_enhanced" {
            let acct = account_id.as_deref().unwrap_or("household");
            let series = self
                .repo
                .fetch_daily_series_with_bands(latest.id, acct, Some(today), Some(end))
                .await?;
            let last = series.last();
            serde_json::json!({
                "computation_id": latest.id.to_string(),
                "variant": model_kind,
                "horizon": horizon,
                "account_id": account_id,
                "end_balance": last.map(|p| p.balance).unwrap_or(0.0),
                "end_balance_p10": last.and_then(|p| p.balance_p10),
                "end_balance_p90": last.and_then(|p| p.balance_p90),
                "series": series.iter().take(30).map(|p| serde_json::json!({
                    "date": p.date.to_string(),
                    "balance": p.balance,
                    "balance_p10": p.balance_p10,
                    "balance_p90": p.balance_p90,
                })).collect::<Vec<_>>(),
            })
        } else {
            let series = if let Some(ref acct) = account_id {
                self.repo
                    .fetch_daily_series(latest.id, acct, Some(today), Some(end))
                    .await?
            } else {
                self.aggregate_daily_balances(latest.id, None).await?
            };

            let points: Vec<serde_json::Value> = series
                .iter()
                .take(30)
                .map(|p| {
                    serde_json::json!({
                        "date": p.date.to_string(),
                        "balance": p.balance,
                    })
                })
                .collect();

            let end_balance = series.last().map(|p| p.balance).unwrap_or(0.0);
            serde_json::json!({
                "computation_id": latest.id.to_string(),
                "variant": "baseline",
                "horizon": horizon,
                "account_id": account_id,
                "end_balance": end_balance,
                "series": points,
            })
        };

        Ok(payload)
    }
}
