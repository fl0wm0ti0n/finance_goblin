use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PnlSnapshotRow {
    pub snapshot_date: NaiveDate,
    pub realized_pnl_eur: f64,
    pub unrealized_pnl_eur: f64,
    pub total_return_pct: Option<f64>,
    pub crypto_value_eur: f64,
    pub payload: serde_json::Value,
}

pub struct PortfolioRepository {
    pool: PgPool,
}

impl PortfolioRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn upsert_pnl_snapshot(
        &self,
        date: NaiveDate,
        run_id: Uuid,
        realized: f64,
        unrealized: f64,
        total_return_pct: Option<f64>,
        crypto_value: f64,
        payload: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO portfolio_pnl_snapshots (
                snapshot_date, sync_run_id, realized_pnl_eur, unrealized_pnl_eur,
                total_return_pct, crypto_value_eur, payload
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (snapshot_date) DO UPDATE SET
                sync_run_id = EXCLUDED.sync_run_id,
                realized_pnl_eur = EXCLUDED.realized_pnl_eur,
                unrealized_pnl_eur = EXCLUDED.unrealized_pnl_eur,
                total_return_pct = EXCLUDED.total_return_pct,
                crypto_value_eur = EXCLUDED.crypto_value_eur,
                payload = EXCLUDED.payload
            "#,
        )
        .bind(date)
        .bind(run_id)
        .bind(realized)
        .bind(unrealized)
        .bind(total_return_pct)
        .bind(crypto_value)
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_pnl_history(&self, days: u32) -> Result<Vec<PnlSnapshotRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT snapshot_date,
                   realized_pnl_eur::float8 AS realized_pnl_eur,
                   unrealized_pnl_eur::float8 AS unrealized_pnl_eur,
                   total_return_pct::float8 AS total_return_pct,
                   crypto_value_eur::float8 AS crypto_value_eur
            FROM portfolio_pnl_snapshots
            WHERE snapshot_date >= CURRENT_DATE - $1::int
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn latest_pnl(&self) -> Result<Option<PnlSnapshotRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT snapshot_date,
                   realized_pnl_eur::float8 AS realized_pnl_eur,
                   unrealized_pnl_eur::float8 AS unrealized_pnl_eur,
                   total_return_pct::float8 AS total_return_pct,
                   crypto_value_eur::float8 AS crypto_value_eur,
                   payload
            FROM portfolio_pnl_snapshots
            ORDER BY snapshot_date DESC LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
    }
}
