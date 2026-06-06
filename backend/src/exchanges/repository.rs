use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::{
    ExchangeFundingEvent, ExchangeHolding, ExchangeSyncState, ExchangeTrade, ExchangeTransfer,
};

pub struct ExchangeRepository {
    pool: PgPool,
}

impl ExchangeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn list_connections(&self) -> Result<Vec<super::types::ConnectionRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, enabled, connection_state, last_sync_at, last_error
            FROM exchange_connections ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_connection_state(
        &self,
        exchange_id: &str,
        state: &str,
        last_error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE exchange_connections
            SET connection_state = $2,
                last_error = $3,
                last_sync_at = CASE WHEN $2 = 'connected' THEN now() ELSE last_sync_at END
            WHERE id = $1
            "#,
        )
        .bind(exchange_id)
        .bind(state)
        .bind(last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_enabled(&self, exchange_id: &str, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE exchange_connections SET enabled = $2 WHERE id = $1")
            .bind(exchange_id)
            .bind(enabled)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_sync_state(&self, exchange_id: &str) -> Result<ExchangeSyncState, sqlx::Error> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT watermarks FROM exchange_sync_state WHERE exchange_id = $1",
        )
        .bind(exchange_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .and_then(|(v,)| serde_json::from_value(v).ok())
            .unwrap_or_default())
    }

    pub async fn update_sync_state(
        &self,
        exchange_id: &str,
        state: &ExchangeSyncState,
    ) -> Result<(), sqlx::Error> {
        let watermarks = serde_json::to_value(state).unwrap_or_default();
        sqlx::query(
            r#"
            INSERT INTO exchange_sync_state (exchange_id, watermarks, updated_at)
            VALUES ($1, $2, now())
            ON CONFLICT (exchange_id) DO UPDATE SET
                watermarks = EXCLUDED.watermarks,
                updated_at = now()
            "#,
        )
        .bind(exchange_id)
        .bind(watermarks)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_holdings(
        &self,
        exchange_id: &str,
        holdings: &[ExchangeHolding],
    ) -> Result<(), sqlx::Error> {
        for h in holdings {
            sqlx::query(
                r#"
                INSERT INTO exchange_holdings (
                    exchange_id, asset, quantity, product_type, payload, synced_at
                ) VALUES ($1, $2, $3, $4, $5, now())
                ON CONFLICT (exchange_id, asset, product_type) DO UPDATE SET
                    quantity = EXCLUDED.quantity,
                    payload = EXCLUDED.payload,
                    synced_at = now()
                "#,
            )
            .bind(exchange_id)
            .bind(&h.asset)
            .bind(h.quantity)
            .bind(&h.product_type)
            .bind(&h.payload)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn update_holding_eur(
        &self,
        exchange_id: &str,
        asset: &str,
        product_type: &str,
        market_value_eur: Option<f64>,
        unrealized_pnl_eur: Option<f64>,
        avg_cost_eur: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE exchange_holdings SET
                market_value_eur = $4,
                unrealized_pnl_eur = $5,
                avg_cost_eur = $6
            WHERE exchange_id = $1 AND asset = $2 AND product_type = $3
            "#,
        )
        .bind(exchange_id)
        .bind(asset)
        .bind(product_type)
        .bind(market_value_eur)
        .bind(unrealized_pnl_eur)
        .bind(avg_cost_eur)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_trades(
        &self,
        exchange_id: &str,
        trades: &[ExchangeTrade],
    ) -> Result<(), sqlx::Error> {
        for t in trades {
            sqlx::query(
                r#"
                INSERT INTO exchange_trades (
                    exchange_id, external_id, symbol, side, quantity, price,
                    fee, fee_asset, realized_pnl, executed_at, payload
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (exchange_id, external_id) DO UPDATE SET
                    quantity = EXCLUDED.quantity,
                    price = EXCLUDED.price,
                    realized_pnl = EXCLUDED.realized_pnl,
                    payload = EXCLUDED.payload
                "#,
            )
            .bind(exchange_id)
            .bind(&t.external_id)
            .bind(&t.symbol)
            .bind(&t.side)
            .bind(t.quantity)
            .bind(t.price)
            .bind(t.fee)
            .bind(&t.fee_asset)
            .bind(t.realized_pnl)
            .bind(t.executed_at)
            .bind(&t.payload)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn upsert_transfers(
        &self,
        exchange_id: &str,
        transfers: &[ExchangeTransfer],
    ) -> Result<(), sqlx::Error> {
        for t in transfers {
            sqlx::query(
                r#"
                INSERT INTO exchange_transfers (
                    exchange_id, external_id, transfer_type, asset, quantity, status, executed_at, payload
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (exchange_id, external_id) DO NOTHING
                "#,
            )
            .bind(exchange_id)
            .bind(&t.external_id)
            .bind(&t.transfer_type)
            .bind(&t.asset)
            .bind(t.quantity)
            .bind(&t.status)
            .bind(t.executed_at)
            .bind(&t.payload)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn upsert_funding(
        &self,
        exchange_id: &str,
        events: &[ExchangeFundingEvent],
    ) -> Result<(), sqlx::Error> {
        for e in events {
            sqlx::query(
                r#"
                INSERT INTO exchange_funding_events (
                    exchange_id, external_id, symbol, amount, asset, event_type, executed_at, payload
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (exchange_id, external_id) DO NOTHING
                "#,
            )
            .bind(exchange_id)
            .bind(&e.external_id)
            .bind(&e.symbol)
            .bind(e.amount)
            .bind(&e.asset)
            .bind(&e.event_type)
            .bind(e.executed_at)
            .bind(&e.payload)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn entity_counts(&self, exchange_id: &str) -> Result<super::types::ExchangeEntityCounts, sqlx::Error> {
        let holdings: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exchange_holdings WHERE exchange_id = $1",
        )
        .bind(exchange_id)
        .fetch_one(&self.pool)
        .await?;

        let trades: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exchange_trades WHERE exchange_id = $1",
        )
        .bind(exchange_id)
        .fetch_one(&self.pool)
        .await?;

        let transfers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exchange_transfers WHERE exchange_id = $1",
        )
        .bind(exchange_id)
        .fetch_one(&self.pool)
        .await?;

        let funding: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exchange_funding_events WHERE exchange_id = $1",
        )
        .bind(exchange_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(super::types::ExchangeEntityCounts {
            holdings,
            trades,
            transfers,
            funding,
        })
    }

    pub async fn load_all_holdings(
        &self,
    ) -> Result<Vec<HoldingRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT exchange_id, asset, quantity::float8 AS quantity, product_type,
                   market_value_eur::float8 AS market_value_eur,
                   unrealized_pnl_eur::float8 AS unrealized_pnl_eur,
                   avg_cost_eur::float8 AS avg_cost_eur
            FROM exchange_holdings
            WHERE quantity > 0
            ORDER BY market_value_eur DESC NULLS LAST
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn load_trades_since(
        &self,
        exchange_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<TradeRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT symbol, side, quantity::float8 AS quantity, price::float8 AS price,
                   fee::float8 AS fee, fee_asset, realized_pnl::float8 AS realized_pnl,
                   executed_at
            FROM exchange_trades
            WHERE exchange_id = $1 AND executed_at >= $2
            ORDER BY executed_at
            "#,
        )
        .bind(exchange_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn prune_trades(&self, retention_days: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM exchange_trades WHERE executed_at < now() - ($1::int || ' days')::interval",
        )
        .bind(retention_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HoldingRow {
    pub exchange_id: String,
    pub asset: String,
    pub quantity: f64,
    pub product_type: String,
    pub market_value_eur: Option<f64>,
    pub unrealized_pnl_eur: Option<f64>,
    pub avg_cost_eur: Option<f64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TradeRow {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub fee: Option<f64>,
    pub fee_asset: Option<String>,
    pub realized_pnl: Option<f64>,
    pub executed_at: DateTime<Utc>,
}
