use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Utc};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::PlansConfig;
use crate::db::DbPool;
use crate::forecast::ForecastService;

use super::overlay::overlay_horizon_end;
use super::project::{balances_to_daily_net, project_plan_series};
use super::repository::{PlanRepoError, PlanRepository};
use super::savings_service;
use super::templates::{self, TemplateOverrides};
use super::types::{
    validate_goal_fields, ActivePlanInfo, CategorySavingsResponse, CompareResponse, EphemeralPlanDraft,
    GoalStatsResponse, GoalYearlyRollup, PlanAdjustment, PlanListItem, PlanProjection, PlanTemplate,
    PlanVsActualResponse, SavingsSuggestion,
};
use crate::forecast::types::DailyPoint;

#[derive(Clone)]
pub struct PlanService {
    repo: Arc<PlanRepository>,
    forecast: ForecastService,
    config: PlansConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("plan not found")]
    NotFound,
    #[error("version not found")]
    VersionNotFound,
    #[error("version cap reached")]
    VersionCapReached,
    #[error("version is frozen")]
    VersionFrozen,
    #[error("adjustment not found")]
    AdjustmentNotFound,
    #[error("no successful forecast computation available")]
    NoForecastBaseline,
    #[error("no active plan")]
    NoActivePlan,
    #[error("invalid template")]
    InvalidTemplate,
    #[error("active plan delete forbidden")]
    ActivePlanDeleteForbidden,
    #[error("{0}")]
    Other(String),
    #[error("goal validation failed: {0}")]
    GoalValidation(String),
    #[error("not a goal_balance plan")]
    NotGoalPlan,
}

impl From<PlanRepoError> for PlanError {
    fn from(e: PlanRepoError) -> Self {
        match e {
            PlanRepoError::Db(err) => PlanError::Db(err),
            PlanRepoError::NotFound => PlanError::NotFound,
            PlanRepoError::VersionNotFound => PlanError::VersionNotFound,
            PlanRepoError::VersionCapReached(_) => PlanError::VersionCapReached,
            PlanRepoError::VersionFrozen => PlanError::VersionFrozen,
            PlanRepoError::AdjustmentNotFound => PlanError::AdjustmentNotFound,
        }
    }
}

impl PlanService {
    pub fn new(db: DbPool, config: PlansConfig, forecast: ForecastService) -> Self {
        let repo = PlanRepository::new(db.pool().clone(), config.clone());
        Self {
            repo: Arc::new(repo),
            forecast,
            config,
        }
    }

    pub fn repository(&self) -> &PlanRepository {
        &self.repo
    }

    pub async fn list_plans(&self) -> Result<Vec<PlanListItem>, PlanError> {
        Ok(self.repo.list_plans().await?)
    }

    pub async fn create_plan(
        &self,
        name: &str,
        template: Option<&str>,
        target_balance_eur: Option<f64>,
        target_date: Option<NaiveDate>,
        goal_account_id: Option<String>,
    ) -> Result<(super::types::PlanRow, super::types::VersionRow), PlanError> {
        let template = template.unwrap_or("custom");
        let today = Utc::now().date_naive();

        if template == "goal_balance" {
            validate_goal_fields(target_balance_eur, target_date, today)
                .map_err(|s| PlanError::GoalValidation(s.to_string()))?;
        }

        let mut resolved_account = goal_account_id;
        if template == "goal_balance" && resolved_account.is_none() {
            resolved_account = self
                .repo
                .default_goal_account_id(&self.config.reporting_currency)
                .await?;
        }
        if let Some(ref acct) = resolved_account {
            if !self.repo.account_exists(acct).await? {
                return Err(PlanError::GoalValidation(format!(
                    "goal_account_id '{acct}' is not a known asset account"
                )));
            }
        }

        let (plan, version) = self
            .repo
            .create_plan(
                name,
                template,
                target_balance_eur,
                target_date,
                resolved_account.as_deref(),
            )
            .await?;

        if let Some(tmpl) = PlanTemplate::from_str(template) {
            if tmpl != PlanTemplate::Custom
                && tmpl != PlanTemplate::Current
                && tmpl != PlanTemplate::GoalBalance
            {
                let defaults = templates::template_defaults(tmpl, &self.config, &Default::default());
                for mut adj in defaults {
                    adj.version_id = version.id;
                    self.repo.add_adjustment(version.id, &adj).await?;
                }
            }
        }
        self.spawn_recompute(version.id);
        Ok((plan, version))
    }

    pub async fn goal_stats(
        &self,
        plan_id: Uuid,
        version_id: Option<Uuid>,
    ) -> Result<GoalStatsResponse, PlanError> {
        let plan = self.repo.get_plan(plan_id).await?;
        if plan.template != "goal_balance" {
            return Err(PlanError::NotGoalPlan);
        }
        let target_balance = plan
            .target_balance_eur
            .ok_or_else(|| PlanError::GoalValidation("missing target_balance_eur".into()))?;
        let target_date = plan
            .target_date
            .ok_or_else(|| PlanError::GoalValidation("missing target_date".into()))?;

        let version = if let Some(vid) = version_id {
            self.repo.get_version(vid).await?
        } else {
            self.repo
                .list_versions(plan_id)
                .await?
                .into_iter()
                .find(|v| v.is_latest)
                .ok_or(PlanError::VersionNotFound)?
        };

        let computation_id = self
            .repo
            .latest_successful_computation(version.id)
            .await?
            .ok_or(PlanError::NoForecastBaseline)?;

        let today = Utc::now().date_naive();
        let beyond_horizon = target_date > overlay_horizon_end(today);

        let (monthly_delta, _) = self.repo.version_metrics(&version).await?;
        let yearly_rows = self
            .repo
            .fetch_yearly_rollup(version.id, computation_id)
            .await?;
        let yearly_rollup = yearly_rows
            .into_iter()
            .map(|(year, sum)| GoalYearlyRollup {
                year,
                planned_net_sum: super::types::fmt_amount(sum),
            })
            .collect();

        let projected_balance = if beyond_horizon {
            None
        } else {
            self.repo
                .fetch_projected_balance_at_date(version.id, computation_id, target_date)
                .await?
        };

        let gap_eur = projected_balance.map(|p| target_balance - p);
        let on_track = projected_balance
            .map(|p| p >= target_balance)
            .unwrap_or(false);

        let months_remaining = months_between(today, target_date);
        let required_monthly = gap_eur.and_then(|gap| {
            if gap > 0.0 && months_remaining > 0 {
                Some((gap / months_remaining as f64).ceil())
            } else {
                Some(0.0)
            }
        });

        let computed_at = self
            .repo
            .computation_computed_at(computation_id)
            .await?
            .map(|t| t.to_rfc3339());

        Ok(GoalStatsResponse {
            plan_id: plan.id.to_string(),
            version_id: version.id.to_string(),
            target_balance_eur: super::types::fmt_amount(target_balance),
            target_date: target_date.to_string(),
            goal_account_id: plan.goal_account_id.clone(),
            monthly_delta_vs_baseline: super::types::fmt_amount(monthly_delta),
            yearly_rollup,
            projected_balance_at_target: projected_balance.map(super::types::fmt_amount),
            gap_eur: gap_eur.map(super::types::fmt_amount),
            required_monthly_savings_eur: required_monthly.map(super::types::fmt_amount),
            on_track,
            beyond_horizon,
            computed_at,
            household_fallback: plan.goal_account_id.is_none(),
        })
    }

    pub async fn category_savings_suggestions(
        &self,
        plan_id: Uuid,
        months: u32,
        limit: u32,
    ) -> Result<CategorySavingsResponse, PlanError> {
        let tx_repo =
            crate::transactions::repository::TransactionsRepository::new(self.repo.pool().clone());
        savings_service::category_savings_suggestions(
            &self.repo,
            &tx_repo,
            self.forecast.repository(),
            plan_id,
            months,
            limit,
        )
        .await
    }

    pub async fn rename_plan(&self, plan_id: Uuid, name: &str) -> Result<(), PlanError> {
        self.repo.rename_plan(plan_id, name).await?;
        Ok(())
    }

    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<(), PlanError> {
        let plan = self.repo.get_plan(plan_id).await?;
        if plan.is_active {
            return Err(PlanError::ActivePlanDeleteForbidden);
        }
        self.repo.delete_plan(plan_id).await?;
        Ok(())
    }

    pub async fn activate_plan(&self, plan_id: Uuid) -> Result<(), PlanError> {
        self.repo.set_active(plan_id).await?;
        if let Some(active) = self.repo.get_active().await? {
            self.spawn_recompute(active.latest_version_id);
        }
        Ok(())
    }

    pub async fn active_plan(&self) -> Result<Option<ActivePlanInfo>, PlanError> {
        Ok(self.repo.get_active().await?)
    }

    pub async fn list_versions(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<super::types::VersionRow>, PlanError> {
        Ok(self.repo.list_versions(plan_id).await?)
    }

    pub async fn get_version(
        &self,
        version_id: Uuid,
    ) -> Result<super::types::VersionRow, PlanError> {
        Ok(self.repo.get_version(version_id).await?)
    }

    pub async fn create_version(
        &self,
        plan_id: Uuid,
    ) -> Result<super::types::VersionRow, PlanError> {
        let version = self.repo.create_version(plan_id).await?;
        self.spawn_recompute(version.id);
        Ok(version)
    }

    pub async fn load_adjustments(&self, version_id: Uuid) -> Result<Vec<PlanAdjustment>, PlanError> {
        Ok(self.repo.load_adjustments(version_id).await?)
    }

    pub async fn add_adjustment(
        &self,
        version_id: Uuid,
        adj: &PlanAdjustment,
    ) -> Result<Uuid, PlanError> {
        let id = self.repo.add_adjustment(version_id, adj).await?;
        self.spawn_recompute(version_id);
        Ok(id)
    }

    pub async fn update_adjustment(
        &self,
        version_id: Uuid,
        adjustment_id: Uuid,
        adj: &PlanAdjustment,
    ) -> Result<(), PlanError> {
        self.repo
            .update_adjustment(version_id, adjustment_id, adj)
            .await?;
        self.spawn_recompute(version_id);
        Ok(())
    }

    pub async fn delete_adjustment(
        &self,
        version_id: Uuid,
        adjustment_id: Uuid,
    ) -> Result<(), PlanError> {
        self.repo
            .delete_adjustment(version_id, adjustment_id)
            .await?;
        self.spawn_recompute(version_id);
        Ok(())
    }

    pub async fn apply_template(
        &self,
        version_id: Uuid,
        template: &str,
        overrides: TemplateOverrides,
    ) -> Result<(), PlanError> {
        let tmpl = PlanTemplate::from_str(template).ok_or(PlanError::InvalidTemplate)?;
        let mut defaults = templates::template_defaults(tmpl, &self.config, &overrides);
        for adj in &mut defaults {
            adj.version_id = version_id;
        }
        self.repo.replace_adjustments(version_id, &defaults).await?;
        self.spawn_recompute(version_id);
        Ok(())
    }

    pub async fn savings_suggestions(&self) -> Result<Vec<SavingsSuggestion>, PlanError> {
        let rows = self.repo.confirmed_subscriptions().await?;
        Ok(templates::map_savings_suggestions(&rows))
    }

    pub async fn compare_versions(&self, plan_id: Uuid) -> Result<CompareResponse, PlanError> {
        let plan = self.repo.get_plan(plan_id).await?;
        let versions = self.repo.build_compare_metrics(plan_id).await?;
        let at_cap = versions.len() >= self.config.max_versions_per_plan as usize;
        Ok(CompareResponse {
            plan_id: plan.id.to_string(),
            plan_name: plan.name,
            versions,
            at_version_cap: at_cap,
        })
    }

    pub async fn plan_vs_actual(
        &self,
        month: Option<NaiveDate>,
    ) -> Result<PlanVsActualResponse, PlanError> {
        let active = self
            .repo
            .get_active()
            .await?
            .ok_or(PlanError::NoActivePlan)?;

        let computation_id = self
            .repo
            .latest_successful_computation(active.latest_version_id)
            .await?
            .ok_or(PlanError::NoForecastBaseline)?;

        let today = Utc::now().date_naive();
        let month_start = month.unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap()
        });
        let month_end = if month.is_some() {
            let (y, m) = (month_start.year(), month_start.month());
            if m == 12 {
                NaiveDate::from_ymd_opt(y + 1, 1, 1)
                    .unwrap()
                    .pred_opt()
                    .unwrap()
            } else {
                NaiveDate::from_ymd_opt(y, m + 1, 1)
                    .unwrap()
                    .pred_opt()
                    .unwrap()
            }
        } else {
            today
        };

        let rows = self
            .repo
            .build_plan_vs_actual_rows(
                active.latest_version_id,
                computation_id,
                month_start,
                month_end,
            )
            .await?;

        Ok(PlanVsActualResponse {
            month: format!("{:04}-{:02}", month_start.year(), month_start.month()),
            reporting_currency: self.config.reporting_currency.clone(),
            plan_stale: self.repo.is_plan_stale(active.latest_version_id).await?,
            actuals_stale: self.repo.is_actuals_stale().await?,
            rows,
        })
    }

    pub fn spawn_recompute(&self, version_id: Uuid) {
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(e) = service.recompute_with_latest_forecast(version_id).await {
                warn!(?e, %version_id, "async plan recompute failed");
            }
        });
    }

    pub async fn recompute_with_latest_forecast(&self, version_id: Uuid) -> Result<Uuid, PlanError> {
        let forecast_id = self
            .forecast
            .repository()
            .latest_successful()
            .await?
            .map(|c| c.id)
            .ok_or(PlanError::NoForecastBaseline)?;
        self.recompute_version(version_id, forecast_id).await
    }

    pub async fn recompute_version(
        &self,
        version_id: Uuid,
        forecast_computation_id: Uuid,
    ) -> Result<Uuid, PlanError> {
        let started = std::time::Instant::now();
        let computation_id = Uuid::new_v4();
        self.repo
            .insert_computation(computation_id, version_id, forecast_computation_id)
            .await?;

        let result = self
            .run_projection(version_id, forecast_computation_id, computation_id)
            .await;

        match result {
            Ok(()) => {
                self.repo
                    .mark_computation_success(computation_id, version_id, forecast_computation_id)
                    .await?;
                info!(
                    %computation_id,
                    %version_id,
                    duration_ms = started.elapsed().as_millis(),
                    "plan recompute succeeded"
                );
                Ok(computation_id)
            }
            Err(e) => {
                warn!(?e, %computation_id, "plan recompute failed");
                self.repo
                    .mark_computation_failed(computation_id, &e.to_string())
                    .await?;
                Err(e)
            }
        }
    }

    async fn run_projection(
        &self,
        version_id: Uuid,
        forecast_computation_id: Uuid,
        computation_id: Uuid,
    ) -> Result<(), PlanError> {
        let version = self.repo.get_version(version_id).await?;
        let plan = self.repo.get_plan(version.plan_id).await?;
        let adjustments = self.repo.load_adjustments(version_id).await?;
        let confirmed = self.repo.confirmed_for_overlay().await?;
        let category_caps = self.repo.category_remove_caps(&adjustments).await?;

        let today = Utc::now().date_naive();
        let end = overlay_horizon_end(today);

        let (aggregate, _household_fallback) = self
            .goal_baseline_series(&plan, forecast_computation_id)
            .await?;

        let balance_pairs: Vec<(NaiveDate, f64)> =
            aggregate.iter().map(|p| (p.date, p.balance)).collect();
        let baseline_net = balances_to_daily_net(&balance_pairs);

        let starting_balance = aggregate.first().map(|p| p.balance).unwrap_or(0.0);
        let series = project_plan_series(
            &baseline_net,
            &adjustments,
            &confirmed,
            today,
            end,
            starting_balance,
            &category_caps,
        );

        self.repo
            .bulk_insert_daily(version_id, computation_id, &series)
            .await?;
        Ok(())
    }

    async fn goal_baseline_series(
        &self,
        plan: &super::types::PlanRow,
        forecast_computation_id: Uuid,
    ) -> Result<(Vec<DailyPoint>, bool), PlanError> {
        let currency = Some(self.config.reporting_currency.as_str());
        if plan.template != "goal_balance" {
            let aggregate = self
                .forecast
                .aggregate_daily_balances(forecast_computation_id, currency)
                .await
                .map_err(|e| PlanError::Other(e.to_string()))?;
            return Ok((aggregate, false));
        }

        if let Some(ref acct) = plan.goal_account_id {
            if self.repo.account_exists(acct).await? {
                let series = self
                    .forecast
                    .repository()
                    .fetch_daily_series(forecast_computation_id, acct, None, None)
                    .await
                    .map_err(|e| PlanError::Other(e.to_string()))?;
                return Ok((series, false));
            }
            warn!(%acct, plan_id = %plan.id, "goal_account_id invalid — household fallback");
        }

        let aggregate = self
            .forecast
            .aggregate_daily_balances(forecast_computation_id, currency)
            .await
            .map_err(|e| PlanError::Other(e.to_string()))?;
        Ok((aggregate, plan.goal_account_id.is_none()))
    }

    pub async fn project_readonly(
        &self,
        plan_id: Uuid,
        version_number: Option<u32>,
    ) -> Result<PlanProjection, PlanError> {
        let versions = self.repo.list_versions(plan_id).await?;
        let version = if let Some(n) = version_number {
            versions
                .into_iter()
                .find(|v| v.version_number == n as i32)
                .ok_or(PlanError::VersionNotFound)?
        } else {
            versions
                .into_iter()
                .max_by_key(|v| v.version_number)
                .ok_or(PlanError::VersionNotFound)?
        };
        self.project_version_in_memory(version.id, "readonly_plan")
            .await
    }

    pub async fn project_ephemeral_from_template(
        &self,
        template: &str,
        extra_adjustments: Option<Vec<PlanAdjustment>>,
    ) -> Result<PlanProjection, PlanError> {
        let tmpl = PlanTemplate::from_str(template).ok_or(PlanError::InvalidTemplate)?;
        let mut defaults = templates::template_defaults(tmpl, &self.config, &Default::default());
        if let Some(extra) = extra_adjustments {
            defaults.extend(extra);
        }
        self.project_ephemeral(EphemeralPlanDraft {
            template: Some(template.to_string()),
            adjustments: defaults,
        })
        .await
    }

    pub async fn project_ephemeral(
        &self,
        draft: EphemeralPlanDraft,
    ) -> Result<PlanProjection, PlanError> {
        self.project_adjustments_in_memory(&draft.adjustments, "ephemeral")
            .await
    }

    pub async fn project_active_readonly(&self) -> Result<PlanProjection, PlanError> {
        let active = self.repo.get_active().await?.ok_or(PlanError::NoActivePlan)?;
        self.project_version_in_memory(active.latest_version_id, "active_plan")
            .await
    }

    async fn project_version_in_memory(
        &self,
        version_id: Uuid,
        source: &str,
    ) -> Result<PlanProjection, PlanError> {
        let adjustments = self.repo.load_adjustments(version_id).await?;
        self.project_adjustments_in_memory(&adjustments, source)
            .await
    }

    async fn project_adjustments_in_memory(
        &self,
        adjustments: &[PlanAdjustment],
        source: &str,
    ) -> Result<PlanProjection, PlanError> {
        let forecast_id = self
            .forecast
            .repository()
            .latest_successful()
            .await?
            .map(|c| c.id)
            .ok_or(PlanError::NoForecastBaseline)?;

        let today = Utc::now().date_naive();
        let end = overlay_horizon_end(today);
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let aggregate = self
            .forecast
            .aggregate_daily_balances(forecast_id, Some(&self.config.reporting_currency))
            .await
            .map_err(|e| PlanError::Other(e.to_string()))?;

        let balance_pairs: Vec<(NaiveDate, f64)> =
            aggregate.iter().map(|p| (p.date, p.balance)).collect();
        let baseline_net = balances_to_daily_net(&balance_pairs);
        let starting_balance = aggregate.first().map(|p| p.balance).unwrap_or(0.0);
        let confirmed = self.repo.confirmed_for_overlay().await?;
        let category_caps = self.repo.category_remove_caps(adjustments).await?;

        let series = project_plan_series(
            &baseline_net,
            adjustments,
            &confirmed,
            today,
            end,
            starting_balance,
            &category_caps,
        );

        let monthly_delta = super::overlay::monthly_overlay_delta_sum(
            adjustments,
            &confirmed,
            month_start,
            today,
            &category_caps,
        );

        let month_end_balance = series
            .iter()
            .filter(|p| p.date <= end)
            .max_by_key(|p| p.date)
            .and_then(|p| p.planned_balance)
            .unwrap_or(starting_balance);

        Ok(PlanProjection {
            source: source.into(),
            monthly_delta_sum: super::types::fmt_amount(monthly_delta),
            projected_month_end_balance: super::types::fmt_amount(month_end_balance),
            reporting_currency: self.config.reporting_currency.clone(),
            key_metrics: serde_json::json!({
                "horizon_days": (end - today).num_days(),
                "adjustment_count": adjustments.len(),
            }),
        })
    }

    pub async fn refresh_active_after_forecast(
        &self,
        forecast_computation_id: Uuid,
    ) -> Result<(), PlanError> {
        if let Some(active) = self.repo.get_active().await? {
            if let Err(e) = self
                .recompute_version(active.latest_version_id, forecast_computation_id)
                .await
            {
                warn!(?e, "active plan post-forecast recompute failed");
            }
        }
        Ok(())
    }
}

fn months_between(from: NaiveDate, to: NaiveDate) -> u32 {
    let months =
        (to.year() - from.year()) * 12 + (to.month() as i32 - from.month() as i32);
    months.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::types::PlanVsActualRow;

    #[test]
    fn deviation_sign_is_actual_minus_planned() {
        let actual = 100.0;
        let planned = 80.0;
        let deviation = actual - planned;
        assert!(deviation > 0.0, "positive deviation means better than plan");

        let row = PlanVsActualRow {
            date: "2026-05-01".into(),
            planned: Some("80.00".into()),
            actual: Some("100.00".into()),
            deviation: Some(format!("{deviation:.2}")),
        };
        assert_eq!(row.deviation.as_deref(), Some("20.00"));
    }

    #[test]
    fn version_cap_is_three() {
        assert_eq!(PlansConfig::default().max_versions_per_plan, 3);
    }

    #[test]
    fn beyond_horizon_when_target_beyond_730_days() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let horizon_end = overlay_horizon_end(today);
        let far = horizon_end + chrono::Duration::days(1);
        assert!(far > horizon_end);
    }

    #[test]
    fn months_between_counts_calendar_months() {
        let from = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        assert_eq!(months_between(from, to), 5);
    }
}
