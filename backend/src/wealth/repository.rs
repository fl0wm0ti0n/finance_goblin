use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::WealthConfig;

use super::types::{AssetAccountRow, SnapshotRow};

pub struct WealthRepository {
    pool: PgPool,
    config: WealthConfig,
}

impl WealthRepository {
    pub fn new(pool: PgPool, config: WealthConfig) -> Self {
        Self { pool, config }
    }

    pub fn config(&self) -> &WealthConfig {
        &self.config
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn load_asset_accounts(&self) -> Result<Vec<AssetAccountRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT firefly_id, name, currency, balance::float8 AS balance,
                   COALESCE(
                     payload->'attributes'->>'account_role',
                     payload->>'account_role'
                   ) AS account_role
            FROM accounts
            WHERE type = 'asset'
              AND COALESCE((payload->>'active')::boolean, true) = true
              AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn last_successful_sync_at(&self) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT finished_at FROM sync_runs
            WHERE status = 'success' ORDER BY finished_at DESC NULLS LAST LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert_snapshot(
        &self,
        date: NaiveDate,
        total: f64,
        mixed_currency: bool,
        account_count: i32,
        payload: &serde_json::Value,
        sync_run_id: Option<Uuid>,
        crypto_value_eur: f64,
        firefly_value_eur: f64,
        total_return_pct: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO net_worth_snapshots (
                snapshot_date, total_eur, mixed_currency, account_count, payload, sync_run_id,
                crypto_value_eur, firefly_value_eur, total_return_pct
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (snapshot_date) DO UPDATE SET
                total_eur = EXCLUDED.total_eur,
                mixed_currency = EXCLUDED.mixed_currency,
                account_count = EXCLUDED.account_count,
                payload = EXCLUDED.payload,
                sync_run_id = EXCLUDED.sync_run_id,
                crypto_value_eur = EXCLUDED.crypto_value_eur,
                firefly_value_eur = EXCLUDED.firefly_value_eur,
                total_return_pct = EXCLUDED.total_return_pct
            "#,
        )
        .bind(date)
        .bind(total)
        .bind(mixed_currency)
        .bind(account_count)
        .bind(payload)
        .bind(sync_run_id)
        .bind(crypto_value_eur)
        .bind(firefly_value_eur)
        .bind(total_return_pct)
        .execute(&self.pool)
        .await?;

        let retention = self.config.snapshot_retention_days as i32;
        if retention > 0 {
            sqlx::query(
                "DELETE FROM net_worth_snapshots WHERE snapshot_date < CURRENT_DATE - $1::int",
            )
            .bind(retention)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn fetch_history(&self, days: u32) -> Result<Vec<SnapshotRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT snapshot_date, total_eur::float8 AS total_eur, mixed_currency, account_count, payload,
                   crypto_value_eur::float8 AS crypto_value_eur,
                   firefly_value_eur::float8 AS firefly_value_eur,
                   total_return_pct::float8 AS total_return_pct
            FROM net_worth_snapshots
            WHERE snapshot_date >= CURRENT_DATE - $1::int
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn load_asset_accounts_includes_negative_balances() {
        const SQL: &str = r#"
            SELECT firefly_id, name, currency, balance::float8 AS balance,
                   COALESCE(
                     payload->'attributes'->>'account_role',
                     payload->>'account_role'
                   ) AS account_role
            FROM accounts
            WHERE type = 'asset'
              AND COALESCE((payload->>'active')::boolean, true) = true
              AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
            ORDER BY name
            "#;
        assert!(
            !SQL.contains(">= 0"),
            "negative-balance asset accounts must remain visible per DEC-0065"
        );
        assert!(
            SQL.contains("payload->'attributes'->>'account_role'"),
            "account_role must read nested attributes path per DEC-0111"
        );
        assert!(
            SQL.contains("COALESCE("),
            "account_role must COALESCE attributes and root fallback per DEC-0111"
        );
    }
}
