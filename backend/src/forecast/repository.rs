use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::{DailyPoint, DailyPointWithBands, MonthlyCashflow, TransactionRow};
use crate::config::ForecastConfig;

#[derive(Debug, sqlx::FromRow)]
pub struct ComputationRow {
    pub id: Uuid,
    pub sync_run_id: Option<Uuid>,
    pub computed_at: DateTime<Utc>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AssetAccount {
    pub firefly_id: String,
    pub name: Option<String>,
    pub currency: Option<String>,
    pub balance: Option<f64>,
}

pub struct ForecastRepository {
    pool: PgPool,
    config: ForecastConfig,
}

impl ForecastRepository {
    pub fn new(pool: PgPool, config: ForecastConfig) -> Self {
        Self { pool, config }
    }

    pub fn config(&self) -> &ForecastConfig {
        &self.config
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn insert_computation(
        &self,
        id: Uuid,
        sync_run_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO forecast_computations (id, sync_run_id, status, model_kind) VALUES ($1, $2, 'running', 'baseline')",
        )
        .bind(id)
        .bind(sync_run_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_ml_computation(
        &self,
        id: Uuid,
        sync_run_id: Uuid,
        paired_baseline_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO forecast_computations (id, sync_run_id, status, model_kind, paired_baseline_id)
            VALUES ($1, $2, 'running', 'ml_enhanced', $3)
            "#,
        )
        .bind(id)
        .bind(sync_run_id)
        .bind(paired_baseline_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_metadata(
        &self,
        id: Uuid,
        metadata: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE forecast_computations SET metadata = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn merge_metadata(
        &self,
        id: Uuid,
        patch: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE forecast_computations
            SET metadata = metadata || $2::jsonb
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(patch)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_success(
        &self,
        id: Uuid,
        metadata: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE forecast_computations SET status = 'success', computed_at = NOW(), metadata = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(
        &self,
        id: Uuid,
        error: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE forecast_computations SET status = 'failed', computed_at = NOW(), error_message = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn bulk_insert_daily(
        &self,
        computation_id: Uuid,
        account_id: &str,
        points: &[DailyPoint],
    ) -> Result<(), sqlx::Error> {
        for chunk in points.chunks(200) {
            for point in chunk {
                let ts = point
                    .date
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                sqlx::query(
                    "INSERT INTO forecast_balance_daily (ts, account_id, computation_id, balance) VALUES ($1, $2, $3, $4)",
                )
                .bind(ts)
                .bind(account_id)
                .bind(computation_id)
                .bind(point.balance)
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn bulk_insert_monthly(
        &self,
        computation_id: Uuid,
        account_id: &str,
        rows: &[MonthlyCashflow],
    ) -> Result<(), sqlx::Error> {
        for row in rows {
            let ts = row
                .month
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let bucket_sources = row
                .bucket_sources
                .as_ref()
                .map(|s| {
                    json!({
                        "income": s.income,
                        "fixed_costs": s.fixed_costs,
                        "variable_costs": s.variable_costs,
                    })
                });
            sqlx::query(
                r#"
                INSERT INTO forecast_cashflow_monthly
                    (ts, account_id, computation_id, income, fixed_costs, variable_costs, free_cashflow,
                     bucket_sources, ai_mapped)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(ts)
            .bind(account_id)
            .bind(computation_id)
            .bind(row.income)
            .bind(row.fixed_costs)
            .bind(row.variable_costs)
            .bind(row.free_cashflow)
            .bind(bucket_sources)
            .bind(row.ai_mapped)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn bulk_insert_daily_with_bands(
        &self,
        computation_id: Uuid,
        account_id: &str,
        points: &[DailyPointWithBands],
    ) -> Result<(), sqlx::Error> {
        for point in points {
            let ts = point.date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            sqlx::query(
                r#"
                INSERT INTO forecast_balance_daily
                    (ts, account_id, computation_id, balance, balance_p10, balance_p90)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(ts)
            .bind(account_id)
            .bind(computation_id)
            .bind(point.balance)
            .bind(point.balance_p10)
            .bind(point.balance_p90)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn bulk_insert_portfolio_weekly(
        &self,
        computation_id: Uuid,
        points: &[super::types::PortfolioWeeklyPoint],
    ) -> Result<(), sqlx::Error> {
        for point in points {
            let ts = point.ts.and_hms_opt(0, 0, 0).unwrap().and_utc();
            sqlx::query(
                r#"
                INSERT INTO forecast_portfolio_weekly
                    (ts, computation_id, value_eur, value_p10, value_p90)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(ts)
            .bind(computation_id)
            .bind(point.value_eur)
            .bind(point.value_p10)
            .bind(point.value_p90)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn latest_successful(&self) -> Result<Option<ComputationRow>, sqlx::Error> {
        self.latest_successful_by_kind("baseline").await
    }

    pub async fn latest_successful_by_kind(
        &self,
        model_kind: &str,
    ) -> Result<Option<ComputationRow>, sqlx::Error> {
        sqlx::query_as::<_, ComputationRow>(
            r#"
            SELECT id, sync_run_id, computed_at, status, error_message, metadata
            FROM forecast_computations
            WHERE status = 'success' AND model_kind = $1
            ORDER BY computed_at DESC
            LIMIT 1
            "#,
        )
        .bind(model_kind)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn fetch_daily_series_with_bands(
        &self,
        computation_id: Uuid,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<DailyPointWithBands>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DailyBandsDbRow>(
            r#"
            SELECT ts, balance::float8 AS balance,
                   balance_p10::float8 AS balance_p10, balance_p90::float8 AS balance_p90
            FROM forecast_balance_daily
            WHERE computation_id = $1 AND account_id = $2
              AND ($3::date IS NULL OR ts::date >= $3)
              AND ($4::date IS NULL OR ts::date <= $4)
            ORDER BY ts
            "#,
        )
        .bind(computation_id)
        .bind(account_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DailyPointWithBands {
                date: r.ts.date_naive(),
                balance: r.balance,
                balance_p10: r.balance_p10,
                balance_p90: r.balance_p90,
            })
            .collect())
    }

    pub async fn fetch_portfolio_weekly(
        &self,
        computation_id: Uuid,
    ) -> Result<Vec<super::types::PortfolioWeeklyPoint>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PortfolioWeeklyDbRow>(
            r#"
            SELECT ts, value_eur::float8 AS value_eur,
                   value_p10::float8 AS value_p10, value_p90::float8 AS value_p90
            FROM forecast_portfolio_weekly
            WHERE computation_id = $1
            ORDER BY ts
            "#,
        )
        .bind(computation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| super::types::PortfolioWeeklyPoint {
                ts: r.ts.date_naive(),
                value_eur: r.value_eur,
                value_p10: r.value_p10,
                value_p90: r.value_p90,
            })
            .collect())
    }

    pub async fn fetch_historical_monthly_net_cashflow(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<(NaiveDate, f64)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (NaiveDate, f64)>(
            r#"
            SELECT date_trunc('month', date)::date AS month, SUM(amount)::float8 AS net
            FROM transactions
            WHERE account_id = $1 AND date IS NOT NULL
            GROUP BY 1
            ORDER BY 1 DESC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().rev().collect())
    }

    pub async fn fetch_historical_crypto_weekly(
        &self,
        limit: i64,
    ) -> Result<Vec<(NaiveDate, f64)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (NaiveDate, f64)>(
            r#"
            SELECT date_trunc('week', snapshot_date)::date AS week, AVG(crypto_value_eur)::float8 AS val
            FROM net_worth_snapshots
            WHERE crypto_value_eur > 0
            GROUP BY 1
            ORDER BY 1 DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().rev().collect())
    }

    pub async fn latest_any(&self) -> Result<Option<ComputationRow>, sqlx::Error> {
        sqlx::query_as::<_, ComputationRow>(
            r#"
            SELECT id, sync_run_id, computed_at, status, error_message, metadata
            FROM forecast_computations
            ORDER BY computed_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_asset_accounts(&self) -> Result<Vec<AssetAccount>, sqlx::Error> {
        sqlx::query_as::<_, AssetAccount>(
            r#"
            SELECT firefly_id, name, currency, balance::float8 AS balance
            FROM accounts
            WHERE type = 'asset'
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_revenue_accounts(&self) -> Result<Vec<AssetAccount>, sqlx::Error> {
        sqlx::query_as::<_, AssetAccount>(
            r#"
            SELECT firefly_id, name, currency, balance::float8 AS balance
            FROM accounts
            WHERE type = 'revenue'
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn fetch_transactions_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<TransactionRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TransactionDbRow>(
            r#"
            SELECT firefly_id, account_id, date, amount::float8 AS amount, description, category_id, payload
            FROM transactions
            WHERE account_id = $1 AND date IS NOT NULL
            ORDER BY date
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let date = r.date?;
                Some(TransactionRow {
                    firefly_id: r.firefly_id,
                    account_id: r.account_id,
                    date,
                    amount: r.amount.unwrap_or(0.0),
                    description: r.description,
                    category_id: r.category_id,
                    payload: r.payload,
                })
            })
            .collect())
    }

    pub async fn category_name_map(&self) -> Result<HashMap<String, String>, sqlx::Error> {
        let rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT firefly_id, name FROM categories",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|(id, name)| name.map(|n| (id, n)))
            .collect())
    }

    pub async fn fetch_daily_series(
        &self,
        computation_id: Uuid,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<DailyPoint>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DailyDbRow>(
            r#"
            SELECT ts, balance::float8 AS balance
            FROM forecast_balance_daily
            WHERE computation_id = $1 AND account_id = $2
              AND ($3::date IS NULL OR ts::date >= $3)
              AND ($4::date IS NULL OR ts::date <= $4)
            ORDER BY ts
            "#,
        )
        .bind(computation_id)
        .bind(account_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DailyPoint {
                date: r.ts.date_naive(),
                balance: r.balance,
            })
            .collect())
    }

    pub async fn fetch_monthly_series(
        &self,
        computation_id: Uuid,
        account_id: &str,
    ) -> Result<Vec<MonthlyCashflow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MonthlyDbRow>(
            r#"
            SELECT ts, income::float8 AS income, fixed_costs::float8 AS fixed_costs,
                   variable_costs::float8 AS variable_costs, free_cashflow::float8 AS free_cashflow,
                   bucket_sources, ai_mapped
            FROM forecast_cashflow_monthly
            WHERE computation_id = $1 AND account_id = $2
            ORDER BY ts
            "#,
        )
        .bind(computation_id)
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let bucket_sources = r.bucket_sources.and_then(|v| {
                    Some(super::types::MonthlyBucketSources {
                        income: v.get("income")?.as_str()?.to_string(),
                        fixed_costs: v.get("fixed_costs")?.as_str()?.to_string(),
                        variable_costs: v.get("variable_costs")?.as_str()?.to_string(),
                    })
                });
                MonthlyCashflow {
                    month: r.ts.date_naive(),
                    income: r.income,
                    fixed_costs: r.fixed_costs,
                    variable_costs: r.variable_costs,
                    free_cashflow: r.free_cashflow,
                    bucket_sources,
                    ai_mapped: r.ai_mapped,
                }
            })
            .collect())
    }

    pub async fn enforce_retention(&self) -> Result<(), sqlx::Error> {
        let keep = self.config.retention_count.max(1) as i64;
        for kind in ["ml_enhanced", "baseline"] {
            let stale_ids: Vec<Uuid> = sqlx::query_scalar(
                r#"
                SELECT id FROM forecast_computations
                WHERE status = 'success' AND model_kind = $1
                ORDER BY computed_at DESC
                OFFSET $2
                "#,
            )
            .bind(kind)
            .bind(keep)
            .fetch_all(&self.pool)
            .await?;

            for id in stale_ids {
                sqlx::query("DELETE FROM forecast_computations WHERE id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub fn is_stale(&self, latest_success: &Option<ComputationRow>, latest_any: &Option<ComputationRow>) -> bool {
        match (latest_success, latest_any) {
            (Some(success), Some(any)) => any.id != success.id && any.status == "failed",
            _ => false,
        }
    }

    pub fn metadata_low_confidence(metadata: &Value) -> bool {
        metadata
            .get("low_confidence")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || metadata
                .get("accounts")
                .and_then(|v| v.as_object())
                .map(|obj| obj.values().any(|v| v.as_bool() == Some(true)))
                .unwrap_or(false)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TransactionDbRow {
    firefly_id: String,
    account_id: Option<String>,
    date: Option<NaiveDate>,
    amount: Option<f64>,
    description: Option<String>,
    category_id: Option<String>,
    payload: Value,
}

#[derive(Debug, sqlx::FromRow)]
struct DailyBandsDbRow {
    ts: DateTime<Utc>,
    balance: f64,
    balance_p10: Option<f64>,
    balance_p90: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct PortfolioWeeklyDbRow {
    ts: DateTime<Utc>,
    value_eur: f64,
    value_p10: Option<f64>,
    value_p90: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct DailyDbRow {
    ts: DateTime<Utc>,
    balance: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct MonthlyDbRow {
    ts: DateTime<Utc>,
    income: f64,
    fixed_costs: f64,
    variable_costs: f64,
    free_cashflow: f64,
    bucket_sources: Option<Value>,
    ai_mapped: bool,
}

pub fn build_metadata(account_flags: &HashMap<String, bool>, balance_warnings: &[Value]) -> Value {
    let any = account_flags.values().any(|v| *v);
    let mut meta = json!({
        "low_confidence": any,
        "accounts": account_flags,
    });
    if !balance_warnings.is_empty() {
        meta["balance_warnings"] = json!(balance_warnings);
    }
    meta
}

/// Returns a balance warning when starting balance is non-positive and tx history exists.
pub fn balance_warning_entry(
    account_id: &str,
    starting_balance: f64,
    tx_count: usize,
) -> Option<Value> {
    if starting_balance <= 0.0 && tx_count > 0 {
        Some(json!({
            "account_id": account_id,
            "starting_balance": starting_balance,
            "reason": "negative_starting_balance",
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;

    #[test]
    fn balance_warning_when_negative_start_with_history() {
        let warning = balance_warning_entry("114", -3395.75, 42).unwrap();
        assert_eq!(warning["account_id"], "114");
        assert_eq!(warning["reason"], "negative_starting_balance");
    }

    #[test]
    fn balance_warning_absent_for_zero_balance_without_txs() {
        assert!(balance_warning_entry("115", 0.0, 0).is_none());
    }

    #[test]
    fn build_metadata_includes_balance_warnings() {
        let flags = HashMap::from([("114".to_string(), false)]);
        let warnings = vec![balance_warning_entry("114", -1.0, 3).unwrap()];
        let meta = build_metadata(&flags, &warnings);
        assert!(meta.get("balance_warnings").is_some());
    }
}
