use std::time::{Duration, Instant};

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::config::{AppConfig, ConfigError};

pub mod bootstrap;

#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_with_retry(config: &AppConfig) -> Result<Self, ConfigError> {
        let url = config.database_url();
        let max = config.database.max_connections;
        let initial = Duration::from_millis(config.database.startup_retry_initial_ms);
        let max_interval = Duration::from_millis(config.database.startup_retry_max_ms);
        let budget = Duration::from_millis(config.database.startup_retry_total_ms);

        let started = Instant::now();
        let mut delay = initial;
        let mut attempt = 0u32;

        loop {
            attempt += 1;
            match PgPoolOptions::new()
                .max_connections(max)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    tracing::info!(attempt, "connected to external PostgreSQL");
                    return Ok(Self { pool });
                }
                Err(err) => {
                    let elapsed = started.elapsed();
                    if elapsed >= budget {
                        tracing::error!(
                            %err,
                            attempt,
                            elapsed_ms = elapsed.as_millis(),
                            "failed to connect to external PostgreSQL after retry budget exhausted. \
                             Verify DATABASE_HOST/PORT/NAME/USER/PASSWORD and network reachability \
                             (use host.docker.internal:host-gateway on Linux Docker)."
                        );
                        return Err(ConfigError::MissingEnv(format!(
                            "database connection failed after ~{}s: {err}",
                            budget.as_secs()
                        )));
                    }
                    tracing::warn!(
                        %err,
                        attempt,
                        retry_in_ms = delay.as_millis(),
                        "database connection failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(max_interval);
                }
            }
        }
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn is_ready(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    pub async fn entity_count(&self, table: &str) -> Result<i64, sqlx::Error> {
        let sql = match table {
            "accounts" => "SELECT COUNT(*)::bigint AS c FROM accounts",
            "transactions" => "SELECT COUNT(*)::bigint AS c FROM transactions",
            "categories" => "SELECT COUNT(*)::bigint AS c FROM categories",
            "budgets" => "SELECT COUNT(*)::bigint AS c FROM budgets",
            "tags" => "SELECT COUNT(*)::bigint AS c FROM tags",
            "piggy_banks" => "SELECT COUNT(*)::bigint AS c FROM piggy_banks",
            _ => return Ok(0),
        };
        let row = sqlx::query(sql).fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>("c"))
    }
}

pub mod repositories {
    use chrono::{DateTime, Utc};
    use serde_json::Value;
    use sqlx::PgPool;
    use uuid::Uuid;

    pub async fn insert_sync_run(
        pool: &PgPool,
        id: Uuid,
        trigger: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'running', $2)",
        )
        .bind(id)
        .bind(trigger)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn finish_sync_run(
        pool: &PgPool,
        id: Uuid,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE sync_runs SET finished_at = NOW(), status = $2, error_message = $3 WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(error)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_cursor(
        pool: &PgPool,
        entity_type: &str,
        synced_at: DateTime<Utc>,
        records: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO sync_cursors (entity_type, last_successful_sync_at, records_synced, last_error)
            VALUES ($1, $2, $3, NULL)
            ON CONFLICT (entity_type) DO UPDATE SET
                last_successful_sync_at = EXCLUDED.last_successful_sync_at,
                records_synced = EXCLUDED.records_synced,
                last_error = NULL
            "#,
        )
        .bind(entity_type)
        .bind(synced_at)
        .bind(records)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_account(
        pool: &PgPool,
        firefly_id: &str,
        account_type: Option<&str>,
        name: Option<&str>,
        iban: Option<&str>,
        currency: Option<&str>,
        balance: Option<f64>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO accounts (firefly_id, type, name, iban, currency, balance, payload, synced_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET
                type = EXCLUDED.type,
                name = EXCLUDED.name,
                iban = EXCLUDED.iban,
                currency = EXCLUDED.currency,
                balance = EXCLUDED.balance,
                payload = EXCLUDED.payload,
                synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(account_type)
        .bind(name)
        .bind(iban)
        .bind(currency)
        .bind(balance)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_transaction(
        pool: &PgPool,
        firefly_id: &str,
        account_id: Option<&str>,
        date: Option<chrono::NaiveDate>,
        amount: Option<f64>,
        description: Option<&str>,
        category_id: Option<&str>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO transactions (firefly_id, account_id, date, amount, description, category_id, payload, synced_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET
                account_id = EXCLUDED.account_id,
                date = EXCLUDED.date,
                amount = EXCLUDED.amount,
                description = EXCLUDED.description,
                category_id = EXCLUDED.category_id,
                payload = EXCLUDED.payload,
                synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(account_id)
        .bind(date)
        .bind(amount)
        .bind(description)
        .bind(category_id)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_category(
        pool: &PgPool,
        firefly_id: &str,
        name: Option<&str>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO categories (firefly_id, name, payload, synced_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET name = EXCLUDED.name, payload = EXCLUDED.payload, synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(name)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_budget(
        pool: &PgPool,
        firefly_id: &str,
        name: Option<&str>,
        amount: Option<f64>,
        period: Option<&str>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO budgets (firefly_id, name, amount, period, payload, synced_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET
                name = EXCLUDED.name, amount = EXCLUDED.amount, period = EXCLUDED.period,
                payload = EXCLUDED.payload, synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(name)
        .bind(amount)
        .bind(period)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_tag(
        pool: &PgPool,
        firefly_id: &str,
        tag: Option<&str>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO tags (firefly_id, tag, payload, synced_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET tag = EXCLUDED.tag, payload = EXCLUDED.payload, synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(tag)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_piggy_bank(
        pool: &PgPool,
        firefly_id: &str,
        name: Option<&str>,
        target_amount: Option<f64>,
        current_amount: Option<f64>,
        payload: &Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO piggy_banks (firefly_id, name, target_amount, current_amount, payload, synced_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (firefly_id) DO UPDATE SET
                name = EXCLUDED.name,
                target_amount = EXCLUDED.target_amount,
                current_amount = EXCLUDED.current_amount,
                payload = EXCLUDED.payload,
                synced_at = NOW()
            "#,
        )
        .bind(firefly_id)
        .bind(name)
        .bind(target_amount)
        .bind(current_amount)
        .bind(payload)
        .execute(pool)
        .await?;
        Ok(())
    }
}
