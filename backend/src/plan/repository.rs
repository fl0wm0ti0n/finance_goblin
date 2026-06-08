use std::collections::HashMap;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::PlansConfig;

use super::overlay::CategoryRemoveCaps;
use super::types::{
    ActivePlanInfo, AdjustmentDirection, AdjustmentRow, AdjustmentTarget, CompareVersionMetrics,
    DailyNetPoint, PlanAdjustment, PlanListItem, PlanRow, PlanVsActualRow, VersionRow,
};
use crate::transactions::repository::TransactionsRepository;
use crate::transactions::types::ExpenseSeriesCategory;

#[derive(Debug, thiserror::Error)]
pub enum PlanRepoError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("plan not found")]
    NotFound,
    #[error("version not found")]
    VersionNotFound,
    #[error("version cap reached (max {0} versions per plan)")]
    VersionCapReached(u32),
    #[error("version is frozen and cannot be edited")]
    VersionFrozen,
    #[error("adjustment not found")]
    AdjustmentNotFound,
}

pub struct PlanRepository {
    pool: PgPool,
    config: PlansConfig,
}

impl PlanRepository {
    pub fn new(pool: PgPool, config: PlansConfig) -> Self {
        Self { pool, config }
    }

    pub fn config(&self) -> &PlansConfig {
        &self.config
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create_plan(
        &self,
        name: &str,
        template: &str,
        target_balance_eur: Option<f64>,
        target_date: Option<NaiveDate>,
        goal_account_id: Option<&str>,
    ) -> Result<(PlanRow, VersionRow), PlanRepoError> {
        let mut tx = self.pool.begin().await?;

        let plan: PlanRow = sqlx::query_as(
            r#"
            INSERT INTO plans (name, template, target_balance_eur, target_date, goal_account_id)
            VALUES ($1, $2::plan_template, $3, $4, $5)
            RETURNING id, name, template::text AS template, is_active,
                      target_balance_eur::float8 AS target_balance_eur,
                      target_date, goal_account_id
            "#,
        )
        .bind(name)
        .bind(template)
        .bind(target_balance_eur)
        .bind(target_date)
        .bind(goal_account_id)
        .fetch_one(&mut *tx)
        .await?;

        let version: VersionRow = sqlx::query_as(
            r#"
            INSERT INTO plan_versions (plan_id, version_number, is_latest)
            VALUES ($1, 1, true)
            RETURNING id, plan_id, version_number, is_latest, frozen_at, baseline_computation_id
            "#,
        )
        .bind(plan.id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok((plan, version))
    }

    pub async fn list_plans(&self) -> Result<Vec<PlanListItem>, PlanRepoError> {
        let rows = sqlx::query_as::<_, PlanListDbRow>(
            r#"
            SELECT
                p.id, p.name, p.template::text AS template, p.is_active,
                v.id AS latest_version_id, v.version_number AS latest_version_number,
                COALESCE(
                    (SELECT fc.computed_at < (
                        SELECT computed_at FROM forecast_computations
                        WHERE status = 'success' ORDER BY computed_at DESC LIMIT 1
                    ) FROM plan_computations pc
                     JOIN forecast_computations fc ON fc.id = pc.forecast_computation_id
                     WHERE pc.version_id = v.id AND pc.status = 'success'
                     ORDER BY pc.computed_at DESC LIMIT 1),
                    true
                ) AS plan_stale
            FROM plans p
            LEFT JOIN plan_versions v ON v.plan_id = p.id AND v.is_latest = true
            ORDER BY p.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PlanListItem {
                id: r.id.to_string(),
                name: r.name,
                template: r.template,
                is_active: r.is_active,
                latest_version_id: r.latest_version_id.map(|id| id.to_string()),
                latest_version_number: r.latest_version_number,
                plan_stale: r.plan_stale,
            })
            .collect())
    }

    pub async fn get_plan(&self, plan_id: Uuid) -> Result<PlanRow, PlanRepoError> {
        sqlx::query_as(
            r#"
            SELECT id, name, template::text AS template, is_active,
                   target_balance_eur::float8 AS target_balance_eur,
                   target_date, goal_account_id
            FROM plans WHERE id = $1
            "#,
        )
        .bind(plan_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PlanRepoError::NotFound)
    }

    pub async fn default_goal_account_id(
        &self,
        reporting_currency: &str,
    ) -> Result<Option<String>, PlanRepoError> {
        Ok(sqlx::query_scalar(
            r#"
            SELECT firefly_id FROM accounts
            WHERE type = 'asset' AND currency = $1 AND COALESCE(balance::float8, 0) > 0
            ORDER BY balance::float8 DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(reporting_currency)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn account_exists(&self, firefly_id: &str) -> Result<bool, PlanRepoError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM accounts WHERE firefly_id = $1 AND type = 'asset'",
        )
        .bind(firefly_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn fetch_yearly_rollup(
        &self,
        version_id: Uuid,
        computation_id: Uuid,
    ) -> Result<Vec<(i32, f64)>, PlanRepoError> {
        let rows: Vec<(i32, f64)> = sqlx::query_as(
            r#"
            SELECT EXTRACT(YEAR FROM ts)::int AS yr,
                   COALESCE(SUM(planned_net::float8), 0)
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
            GROUP BY 1
            ORDER BY 1
            "#,
        )
        .bind(version_id)
        .bind(computation_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn fetch_projected_balance_at_date(
        &self,
        version_id: Uuid,
        computation_id: Uuid,
        target_date: NaiveDate,
    ) -> Result<Option<f64>, PlanRepoError> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT planned_balance::float8
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
              AND ts::date <= $3::date
            ORDER BY ts DESC
            LIMIT 1
            "#,
        )
        .bind(version_id)
        .bind(computation_id)
        .bind(target_date)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(b,)| b))
    }

    pub async fn computation_computed_at(
        &self,
        computation_id: Uuid,
    ) -> Result<Option<DateTime<Utc>>, PlanRepoError> {
        Ok(sqlx::query_scalar(
            "SELECT computed_at FROM plan_computations WHERE id = $1",
        )
        .bind(computation_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn rename_plan(&self, plan_id: Uuid, name: &str) -> Result<(), PlanRepoError> {
        let result = sqlx::query(
            "UPDATE plans SET name = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(plan_id)
        .bind(name)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(PlanRepoError::NotFound);
        }
        Ok(())
    }

    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<(), PlanRepoError> {
        let result = sqlx::query("DELETE FROM plans WHERE id = $1")
            .bind(plan_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(PlanRepoError::NotFound);
        }
        Ok(())
    }

    pub async fn list_versions(&self, plan_id: Uuid) -> Result<Vec<VersionRow>, PlanRepoError> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, plan_id, version_number, is_latest, frozen_at, baseline_computation_id
            FROM plan_versions WHERE plan_id = $1 ORDER BY version_number
            "#,
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn get_version(&self, version_id: Uuid) -> Result<VersionRow, PlanRepoError> {
        sqlx::query_as(
            r#"
            SELECT id, plan_id, version_number, is_latest, frozen_at, baseline_computation_id
            FROM plan_versions WHERE id = $1
            "#,
        )
        .bind(version_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PlanRepoError::VersionNotFound)
    }

    pub async fn create_version(&self, plan_id: Uuid) -> Result<VersionRow, PlanRepoError> {
        let mut tx = self.pool.begin().await?;

        let current: Option<VersionRow> = sqlx::query_as(
            r#"
            SELECT id, plan_id, version_number, is_latest, frozen_at, baseline_computation_id
            FROM plan_versions WHERE plan_id = $1 AND is_latest = true FOR UPDATE
            "#,
        )
        .bind(plan_id)
        .fetch_optional(&mut *tx)
        .await?;

        let current = current.ok_or(PlanRepoError::NotFound)?;
        let next_num = current.version_number + 1;
        if next_num as u32 > self.config.max_versions_per_plan {
            return Err(PlanRepoError::VersionCapReached(
                self.config.max_versions_per_plan,
            ));
        }

        sqlx::query(
            "UPDATE plan_versions SET is_latest = false, frozen_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(current.id)
        .execute(&mut *tx)
        .await?;

        let adjustments = self.load_adjustments_in_tx(&mut tx, current.id).await?;
        let new_version: VersionRow = sqlx::query_as(
            r#"
            INSERT INTO plan_versions (plan_id, version_number, is_latest)
            VALUES ($1, $2, true)
            RETURNING id, plan_id, version_number, is_latest, frozen_at, baseline_computation_id
            "#,
        )
        .bind(plan_id)
        .bind(next_num)
        .fetch_one(&mut *tx)
        .await?;

        for adj in adjustments {
            sqlx::query(
                r#"
                INSERT INTO plan_adjustments (
                    version_id, direction, amount, frequency, target_type,
                    target_key, label, effective_from, effective_to, sort_order
                )
                VALUES ($1, $2::plan_adjustment_direction, $3, $4::plan_adjustment_frequency,
                        $5::plan_adjustment_target, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(new_version.id)
            .bind(adj.direction.as_str())
            .bind(adj.amount)
            .bind(adj.frequency.as_str())
            .bind(adj.target_type.as_str())
            .bind(&adj.target_key)
            .bind(&adj.label)
            .bind(adj.effective_from)
            .bind(adj.effective_to)
            .bind(adj.sort_order)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(new_version)
    }

    async fn load_adjustments_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        version_id: Uuid,
    ) -> Result<Vec<PlanAdjustment>, PlanRepoError> {
        let rows: Vec<AdjustmentRow> = sqlx::query_as(
            r#"
            SELECT id, version_id, direction::text AS direction, amount::float8 AS amount,
                   frequency::text AS frequency, target_type::text AS target_type,
                   target_key, label, effective_from, effective_to, sort_order
            FROM plan_adjustments WHERE version_id = $1 ORDER BY sort_order, id
            "#,
        )
        .bind(version_id)
        .fetch_all(&mut **tx)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_adjustment()).collect())
    }

    pub async fn load_adjustments(&self, version_id: Uuid) -> Result<Vec<PlanAdjustment>, PlanRepoError> {
        let rows: Vec<AdjustmentRow> = sqlx::query_as(
            r#"
            SELECT id, version_id, direction::text AS direction, amount::float8 AS amount,
                   frequency::text AS frequency, target_type::text AS target_type,
                   target_key, label, effective_from, effective_to, sort_order
            FROM plan_adjustments WHERE version_id = $1 ORDER BY sort_order, id
            "#,
        )
        .bind(version_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_adjustment()).collect())
    }

    async fn ensure_editable(&self, version_id: Uuid) -> Result<VersionRow, PlanRepoError> {
        let version = self.get_version(version_id).await?;
        if version.frozen_at.is_some() || !version.is_latest {
            return Err(PlanRepoError::VersionFrozen);
        }
        Ok(version)
    }

    pub async fn replace_adjustments(
        &self,
        version_id: Uuid,
        adjustments: &[PlanAdjustment],
    ) -> Result<(), PlanRepoError> {
        self.ensure_editable(version_id).await?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM plan_adjustments WHERE version_id = $1")
            .bind(version_id)
            .execute(&mut *tx)
            .await?;
        for adj in adjustments {
            self.insert_adjustment_in_tx(&mut tx, version_id, adj).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn add_adjustment(
        &self,
        version_id: Uuid,
        adj: &PlanAdjustment,
    ) -> Result<Uuid, PlanRepoError> {
        self.ensure_editable(version_id).await?;
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO plan_adjustments (
                version_id, direction, amount, frequency, target_type,
                target_key, label, effective_from, effective_to, sort_order
            )
            VALUES ($1, $2::plan_adjustment_direction, $3, $4::plan_adjustment_frequency,
                    $5::plan_adjustment_target, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
        )
        .bind(version_id)
        .bind(adj.direction.as_str())
        .bind(adj.amount)
        .bind(adj.frequency.as_str())
        .bind(adj.target_type.as_str())
        .bind(&adj.target_key)
        .bind(&adj.label)
        .bind(adj.effective_from)
        .bind(adj.effective_to)
        .bind(adj.sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn insert_adjustment_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        version_id: Uuid,
        adj: &PlanAdjustment,
    ) -> Result<(), PlanRepoError> {
        sqlx::query(
            r#"
            INSERT INTO plan_adjustments (
                version_id, direction, amount, frequency, target_type,
                target_key, label, effective_from, effective_to, sort_order
            )
            VALUES ($1, $2::plan_adjustment_direction, $3, $4::plan_adjustment_frequency,
                    $5::plan_adjustment_target, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(version_id)
        .bind(adj.direction.as_str())
        .bind(adj.amount)
        .bind(adj.frequency.as_str())
        .bind(adj.target_type.as_str())
        .bind(&adj.target_key)
        .bind(&adj.label)
        .bind(adj.effective_from)
        .bind(adj.effective_to)
        .bind(adj.sort_order)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn update_adjustment(
        &self,
        version_id: Uuid,
        adjustment_id: Uuid,
        adj: &PlanAdjustment,
    ) -> Result<(), PlanRepoError> {
        self.ensure_editable(version_id).await?;
        let result = sqlx::query(
            r#"
            UPDATE plan_adjustments SET
                direction = $3::plan_adjustment_direction,
                amount = $4,
                frequency = $5::plan_adjustment_frequency,
                target_type = $6::plan_adjustment_target,
                target_key = $7,
                label = $8,
                effective_from = $9,
                effective_to = $10,
                sort_order = $11,
                updated_at = NOW()
            WHERE id = $2 AND version_id = $1
            "#,
        )
        .bind(version_id)
        .bind(adjustment_id)
        .bind(adj.direction.as_str())
        .bind(adj.amount)
        .bind(adj.frequency.as_str())
        .bind(adj.target_type.as_str())
        .bind(&adj.target_key)
        .bind(&adj.label)
        .bind(adj.effective_from)
        .bind(adj.effective_to)
        .bind(adj.sort_order)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(PlanRepoError::AdjustmentNotFound);
        }
        Ok(())
    }

    pub async fn delete_adjustment(
        &self,
        version_id: Uuid,
        adjustment_id: Uuid,
    ) -> Result<(), PlanRepoError> {
        self.ensure_editable(version_id).await?;
        let result = sqlx::query(
            "DELETE FROM plan_adjustments WHERE id = $2 AND version_id = $1",
        )
        .bind(version_id)
        .bind(adjustment_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(PlanRepoError::AdjustmentNotFound);
        }
        Ok(())
    }

    pub async fn set_active(&self, plan_id: Uuid) -> Result<(), PlanRepoError> {
        let mut tx = self.pool.begin().await?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM plans WHERE id = $1)")
            .bind(plan_id)
            .fetch_one(&mut *tx)
            .await?;
        if !exists {
            return Err(PlanRepoError::NotFound);
        }

        sqlx::query("UPDATE plans SET is_active = false, updated_at = NOW() WHERE is_active = true")
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE plans SET is_active = true, updated_at = NOW() WHERE id = $1")
            .bind(plan_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_active(&self) -> Result<Option<ActivePlanInfo>, PlanRepoError> {
        let row: Option<ActivePlanDbRow> = sqlx::query_as(
            r#"
            SELECT p.id AS plan_id, p.name AS plan_name,
                   v.id AS latest_version_id, v.version_number AS latest_version_number
            FROM plans p
            JOIN plan_versions v ON v.plan_id = p.id AND v.is_latest = true
            WHERE p.is_active = true
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ActivePlanInfo {
            plan_id: r.plan_id,
            plan_name: r.plan_name,
            latest_version_id: r.latest_version_id,
            latest_version_number: r.latest_version_number,
        }))
    }

    pub async fn insert_computation(
        &self,
        id: Uuid,
        version_id: Uuid,
        forecast_computation_id: Uuid,
    ) -> Result<(), PlanRepoError> {
        sqlx::query(
            r#"
            INSERT INTO plan_computations (id, version_id, forecast_computation_id, status)
            VALUES ($1, $2, $3, 'running')
            "#,
        )
        .bind(id)
        .bind(version_id)
        .bind(forecast_computation_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_computation_success(
        &self,
        id: Uuid,
        version_id: Uuid,
        forecast_computation_id: Uuid,
    ) -> Result<(), PlanRepoError> {
        sqlx::query(
            r#"
            UPDATE plan_computations SET status = 'success', computed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE plan_versions SET baseline_computation_id = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(version_id)
        .bind(forecast_computation_id)
        .execute(&self.pool)
        .await?;

        self.enforce_retention(version_id).await?;
        Ok(())
    }

    pub async fn mark_computation_failed(
        &self,
        id: Uuid,
        error: &str,
    ) -> Result<(), PlanRepoError> {
        sqlx::query(
            "UPDATE plan_computations SET status = 'failed', computed_at = NOW(), error_message = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn enforce_retention(&self, version_id: Uuid) -> Result<(), PlanRepoError> {
        let keep = self.config.computation_retention_per_version as i64;
        let stale: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM plan_computations
            WHERE version_id = $1 AND status = 'success'
            ORDER BY computed_at DESC
            OFFSET $2
            "#,
        )
        .bind(version_id)
        .bind(keep)
        .fetch_all(&self.pool)
        .await?;

        for id in stale {
            sqlx::query("DELETE FROM plan_computations WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn bulk_insert_daily(
        &self,
        version_id: Uuid,
        computation_id: Uuid,
        series: &[DailyNetPoint],
    ) -> Result<(), PlanRepoError> {
        for chunk in series.chunks(200) {
            for point in chunk {
                let ts = point
                    .date
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                sqlx::query(
                    r#"
                    INSERT INTO plan_daily_cashflow (ts, version_id, computation_id, planned_net, planned_balance)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(ts)
                .bind(version_id)
                .bind(computation_id)
                .bind(point.planned_net)
                .bind(point.planned_balance)
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn latest_successful_computation(
        &self,
        version_id: Uuid,
    ) -> Result<Option<Uuid>, PlanRepoError> {
        Ok(sqlx::query_scalar(
            r#"
            SELECT id FROM plan_computations
            WHERE version_id = $1 AND status = 'success'
            ORDER BY computed_at DESC LIMIT 1
            "#,
        )
        .bind(version_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn fetch_planned_series(
        &self,
        version_id: Uuid,
        computation_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<HashMap<NaiveDate, f64>, PlanRepoError> {
        let rows: Vec<(DateTime<Utc>, f64)> = sqlx::query_as(
            r#"
            SELECT ts, planned_net::float8 AS planned_net
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
              AND ts >= $3::date AND ts <= ($4::date + INTERVAL '1 day')
            ORDER BY ts
            "#,
        )
        .bind(version_id)
        .bind(computation_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(ts, net)| (ts.date_naive(), net))
            .collect())
    }

    pub async fn fetch_actual_daily_net(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<HashMap<NaiveDate, f64>, PlanRepoError> {
        let rows: Vec<(NaiveDate, f64)> = sqlx::query_as(
            r#"
            SELECT t.date AS day, SUM(t.amount::float8) AS actual_net
            FROM transactions t
            JOIN accounts a ON a.firefly_id = t.account_id
            WHERE a.type = 'asset'
              AND COALESCE(t.payload->'attributes'->>'type', '') != 'transfer'
              AND t.date >= $1 AND t.date <= $2
            GROUP BY t.date
            ORDER BY t.date
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().collect())
    }

    pub async fn is_plan_stale(&self, version_id: Uuid) -> Result<bool, PlanRepoError> {
        let stale: bool = sqlx::query_scalar(
            r#"
            SELECT COALESCE(
                (SELECT pc.computed_at < (
                    SELECT computed_at FROM forecast_computations
                    WHERE status = 'success' ORDER BY computed_at DESC LIMIT 1
                )
                FROM plan_computations pc
                WHERE pc.version_id = $1 AND pc.status = 'success'
                ORDER BY pc.computed_at DESC LIMIT 1),
                true
            )
            "#,
        )
        .bind(version_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(stale)
    }

    pub async fn is_actuals_stale(&self) -> Result<bool, PlanRepoError> {
        let stale: bool = sqlx::query_scalar(
            r#"
            SELECT COALESCE(
                (SELECT last_successful_sync_at < NOW() - INTERVAL '25 hours'
                 FROM sync_cursors WHERE entity_type = 'transactions'),
                true
            )
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(stale)
    }

    pub async fn confirmed_subscriptions(
        &self,
    ) -> Result<Vec<(Uuid, String, String, f64, i32)>, PlanRepoError> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, payee_key, display_name, current_amount::float8, interval_days
            FROM subscription_patterns WHERE status = 'confirmed'
            ORDER BY display_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn confirmed_for_overlay(
        &self,
    ) -> Result<Vec<super::types::ConfirmedSubscription>, PlanRepoError> {
        let rows: Vec<(String, f64, i32)> = sqlx::query_as(
            r#"
            SELECT payee_key, current_amount::float8, interval_days
            FROM subscription_patterns WHERE status = 'confirmed'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(payee_key, amount, interval_days)| super::types::ConfirmedSubscription {
                payee_key,
                amount,
                interval_days: interval_days as i64,
            })
            .collect())
    }

    pub async fn version_metrics(
        &self,
        version: &VersionRow,
    ) -> Result<(f64, f64), PlanRepoError> {
        let computation_id = match self.latest_successful_computation(version.id).await? {
            Some(id) => id,
            None => return Ok((0.0, 0.0)),
        };

        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
        let adjustments = self.load_adjustments(version.id).await?;
        let confirmed = self.confirmed_for_overlay().await?;
        let category_caps = self.category_remove_caps(&adjustments).await?;
        let monthly_delta = super::overlay::monthly_overlay_delta_sum(
            &adjustments,
            &confirmed,
            month_start,
            today,
            &category_caps,
        );

        let balance_rows: Vec<(NaiveDate, f64)> = sqlx::query_as(
            r#"
            SELECT ts::date AS day, planned_balance::float8 AS bal
            FROM plan_daily_cashflow
            WHERE version_id = $1 AND computation_id = $2
              AND ts >= $3::date AND ts <= ($4::date + INTERVAL '1 day')
            ORDER BY ts DESC LIMIT 1
            "#,
        )
        .bind(version.id)
        .bind(computation_id)
        .bind(month_start)
        .bind(today)
        .fetch_all(&self.pool)
        .await?;

        let month_end_balance = balance_rows.first().map(|(_, b)| *b).unwrap_or(0.0);
        Ok((monthly_delta, month_end_balance))
    }

    pub async fn build_compare_metrics(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<CompareVersionMetrics>, PlanRepoError> {
        let versions = self.list_versions(plan_id).await?;
        let mut metrics = Vec::new();
        for v in versions {
            let (delta, balance) = self.version_metrics(&v).await?;
            metrics.push(CompareVersionMetrics {
                version_id: v.id.to_string(),
                version_number: v.version_number,
                frozen: v.frozen_at.is_some(),
                monthly_delta_sum: super::types::fmt_amount(delta),
                projected_month_end_balance: super::types::fmt_amount(balance),
            });
        }
        Ok(metrics)
    }

    pub async fn category_remove_caps(
        &self,
        adjustments: &[PlanAdjustment],
    ) -> Result<CategoryRemoveCaps, PlanRepoError> {
        let tx_repo = TransactionsRepository::new(self.pool.clone());
        let today = Utc::now().date_naive();
        let (start, end) = last_n_calendar_months_window(today, 3);
        let mut caps = CategoryRemoveCaps::new();

        for adj in adjustments {
            if adj.target_type != AdjustmentTarget::Category
                || adj.direction != AdjustmentDirection::RemoveOutflow
            {
                continue;
            }
            let Some(cat) = &adj.target_key else {
                continue;
            };
            if caps.contains_key(cat) {
                continue;
            }
            let months = tx_repo
                .expense_series_by_month(ExpenseSeriesCategory::MirrorId(cat), start, end)
                .await?;
            let total: f64 = months.iter().map(|m| m.outflow_eur).sum();
            let avg = if months.is_empty() {
                0.0
            } else {
                total / months.len() as f64
            };
            caps.insert(cat.clone(), avg);
        }
        Ok(caps)
    }

    pub async fn build_plan_vs_actual_rows(
        &self,
        version_id: Uuid,
        computation_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<PlanVsActualRow>, PlanRepoError> {
        let planned = self
            .fetch_planned_series(version_id, computation_id, from, to)
            .await?;
        let actual = self.fetch_actual_daily_net(from, to).await?;

        let mut dates: Vec<NaiveDate> = planned.keys().chain(actual.keys()).copied().collect();
        dates.sort();
        dates.dedup();
        dates.retain(|d| *d >= from && *d <= to);

        Ok(dates
            .into_iter()
            .map(|date| {
                let p = planned.get(&date).copied();
                let a = actual.get(&date).copied();
                let deviation = match (a, p) {
                    (Some(act), Some(pl)) => Some(act - pl),
                    _ => None,
                };
                PlanVsActualRow {
                    date: date.to_string(),
                    planned: p.map(super::types::fmt_amount),
                    actual: a.map(super::types::fmt_amount),
                    deviation: deviation.map(super::types::fmt_amount),
                }
            })
            .collect())
    }
}

fn last_n_calendar_months_window(today: NaiveDate, months: u32) -> (NaiveDate, NaiveDate) {
    let total = today.year() as i32 * 12 + today.month() as i32 - 1 - months as i32 + 1;
    let y = total.div_euclid(12);
    let m = (total.rem_euclid(12) + 1) as u32;
    let start = NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(today);
    (start, today)
}

#[derive(Debug, sqlx::FromRow)]
struct PlanListDbRow {
    id: Uuid,
    name: String,
    template: String,
    is_active: bool,
    latest_version_id: Option<Uuid>,
    latest_version_number: Option<i32>,
    plan_stale: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct ActivePlanDbRow {
    plan_id: Uuid,
    plan_name: String,
    latest_version_id: Uuid,
    latest_version_number: i32,
}
