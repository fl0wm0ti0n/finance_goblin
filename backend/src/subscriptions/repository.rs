use std::collections::HashSet;

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::SubscriptionsConfig;
use crate::forecast::types::TransactionRow;
use crate::recurrence::RecurrenceGroup;

use super::types::{AlertRow, PatternDetailRow, PatternRow, PriceEventRow};

pub struct SubscriptionRepository {
    pool: PgPool,
    config: SubscriptionsConfig,
}

impl SubscriptionRepository {
    pub fn new(pool: PgPool, config: SubscriptionsConfig) -> Self {
        Self { pool, config }
    }

    pub fn config(&self) -> &SubscriptionsConfig {
        &self.config
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn load_expense_transactions(
        &self,
        window_days: i64,
    ) -> Result<Vec<TransactionRow>, sqlx::Error> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(window_days);
        let rows = sqlx::query_as::<_, TransactionDbRow>(
            r#"
            SELECT firefly_id, account_id, date, amount::float8 AS amount, description, category_id, payload
            FROM transactions
            WHERE date >= $1 AND amount < 0
            ORDER BY date ASC
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn load_rejection_fingerprints(&self) -> Result<HashSet<String>, sqlx::Error> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT fingerprint FROM subscription_rejections")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Rejections that suppress forecast recurring — subscription kind only (BUG-0012).
    pub async fn load_forecast_excluded_rejections(&self) -> Result<HashSet<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT sr.fingerprint
            FROM subscription_rejections sr
            INNER JOIN subscription_patterns sp ON sp.fingerprint = sr.fingerprint
            WHERE sp.kind = 'subscription'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn load_confirmed_fingerprints(&self) -> Result<HashSet<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT fingerprint FROM subscription_patterns WHERE status = 'confirmed'",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn load_confirmed_for_forecast(
        &self,
    ) -> Result<Vec<super::types::ConfirmedRecurring>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ConfirmedDbRow>(
            r#"
            SELECT payee_key, current_amount::float8 AS amount, interval_days, fingerprint
            FROM subscription_patterns
            WHERE status = 'confirmed'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| super::types::ConfirmedRecurring {
                payee_key: r.payee_key,
                amount: r.amount,
                interval_days: r.interval_days as i64,
                fingerprint: r.fingerprint,
            })
            .collect())
    }

    pub async fn upsert_pending_pattern(
        &self,
        group: &RecurrenceGroup,
        fingerprint: &str,
        kind: &str,
        sync_run_id: Uuid,
    ) -> Result<Uuid, sqlx::Error> {
        let first = group.transaction_dates.first().copied().unwrap_or_else(|| Utc::now().date_naive());
        let last = group.transaction_dates.last().copied().unwrap_or(first);

        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO subscription_patterns (
                fingerprint, status, kind, payee_key, display_name, interval_days,
                current_amount, confidence_pct, first_seen_at, last_seen_at, detection_run_id
            )
            VALUES ($1, 'pending', $2::subscription_kind, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (fingerprint) DO UPDATE SET
                kind = EXCLUDED.kind,
                payee_key = EXCLUDED.payee_key,
                display_name = EXCLUDED.display_name,
                interval_days = EXCLUDED.interval_days,
                current_amount = EXCLUDED.current_amount,
                confidence_pct = GREATEST(subscription_patterns.confidence_pct, EXCLUDED.confidence_pct),
                last_seen_at = GREATEST(subscription_patterns.last_seen_at, EXCLUDED.last_seen_at),
                detection_run_id = EXCLUDED.detection_run_id,
                updated_at = NOW(),
                status = CASE
                    WHEN subscription_patterns.status IN ('confirmed', 'rejected') THEN subscription_patterns.status
                    ELSE 'pending'
                END
            RETURNING id
            "#,
        )
        .bind(fingerprint)
        .bind(kind)
        .bind(&group.payee_key)
        .bind(&group.display_name)
        .bind(group.interval_days as i32)
        .bind(group.median_amount)
        .bind(group.confidence_pct)
        .bind(first)
        .bind(last)
        .bind(sync_run_id)
        .fetch_one(&self.pool)
        .await?;

        self.link_transactions(&id, &group.transaction_ids).await?;
        Ok(id)
    }

    pub async fn link_transactions(
        &self,
        pattern_id: &Uuid,
        transaction_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        for tx_id in transaction_ids {
            sqlx::query(
                r#"
                INSERT INTO subscription_pattern_transactions (pattern_id, transaction_firefly_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(pattern_id)
            .bind(tx_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn insert_rejection(
        &self,
        fingerprint: &str,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO subscription_rejections (fingerprint, reason)
            VALUES ($1, $2)
            ON CONFLICT (fingerprint) DO NOTHING
            "#,
        )
        .bind(fingerprint)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn append_price_event(
        &self,
        pattern_id: Uuid,
        event_type: &str,
        amount: f64,
        previous_amount: Option<f64>,
        delta_pct: Option<f64>,
        interval_days: Option<i32>,
        occurred_at: NaiveDate,
        sync_run_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO subscription_price_events (
                pattern_id, event_type, amount, previous_amount, delta_pct,
                interval_days, occurred_at, sync_run_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(pattern_id)
        .bind(event_type)
        .bind(amount)
        .bind(previous_amount)
        .bind(delta_pct)
        .bind(interval_days)
        .bind(occurred_at)
        .bind(sync_run_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_alert(
        &self,
        pattern_id: Option<Uuid>,
        alert_type: &str,
        title: &str,
        body: Option<&str>,
        sync_run_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO subscription_alerts (pattern_id, alert_type, title, body, sync_run_id)
            VALUES ($1, $2::subscription_alert_type, $3, $4, $5)
            "#,
        )
        .bind(pattern_id)
        .bind(alert_type)
        .bind(title)
        .bind(body)
        .bind(sync_run_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_patterns(
        &self,
        status: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<PatternRow>, sqlx::Error> {
        let mut sql = String::from(
            r#"
            SELECT id, fingerprint, status::text, kind::text, payee_key, display_name,
                   interval_days, current_amount::float8 AS current_amount, confidence_pct, first_seen_at, last_seen_at,
                   confirmed_at, rejected_at, created_at, updated_at
            FROM subscription_patterns
            WHERE 1=1
            "#,
        );
        if status.is_some() {
            sql.push_str(" AND status = $1::subscription_status");
        }
        if kind.is_some() {
            let idx = if status.is_some() { 2 } else { 1 };
            sql.push_str(&format!(" AND kind = ${idx}::subscription_kind"));
        }
        sql.push_str(" ORDER BY last_seen_at DESC");

        let mut q = sqlx::query_as::<_, PatternRow>(&sql);
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(k) = kind {
            q = q.bind(k);
        }
        q.fetch_all(&self.pool).await
    }

    pub async fn get_pattern(&self, id: Uuid) -> Result<Option<PatternDetailRow>, sqlx::Error> {
        sqlx::query_as::<_, PatternDetailRow>(
            r#"
            SELECT p.id, p.fingerprint, p.status::text, p.kind::text, p.payee_key, p.display_name,
                   p.interval_days, p.current_amount::float8 AS current_amount, p.confidence_pct, p.first_seen_at, p.last_seen_at,
                   p.confirmed_at, p.rejected_at,
                   (SELECT COUNT(*)::bigint FROM subscription_pattern_transactions t WHERE t.pattern_id = p.id) AS transaction_count
            FROM subscription_patterns p
            WHERE p.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn confirm_pattern(
        &self,
        id: Uuid,
        kind: Option<&str>,
    ) -> Result<Option<PatternRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, PatternRow>(
            r#"
            UPDATE subscription_patterns
            SET status = 'confirmed',
                kind = COALESCE($2::subscription_kind, kind),
                confirmed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            RETURNING id, fingerprint, status::text, kind::text, payee_key, display_name,
                      interval_days, current_amount::float8 AS current_amount, confidence_pct, first_seen_at, last_seen_at,
                      confirmed_at, rejected_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(kind)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn reject_pattern(
        &self,
        id: Uuid,
        reason: Option<&str>,
    ) -> Result<Option<PatternRow>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, PatternRow>(
            r#"
            UPDATE subscription_patterns
            SET status = 'rejected', rejected_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            RETURNING id, fingerprint, status::text, kind::text, payee_key, display_name,
                      interval_days, current_amount::float8 AS current_amount, confidence_pct, first_seen_at, last_seen_at,
                      confirmed_at, rejected_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(ref pattern) = row {
            sqlx::query(
                "INSERT INTO subscription_rejections (fingerprint, reason) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(&pattern.fingerprint)
            .bind(reason)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(row)
    }

    pub async fn mark_inactive(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE subscription_patterns SET status = 'inactive', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_confirmed_patterns(&self) -> Result<Vec<PatternRow>, sqlx::Error> {
        self.list_patterns(Some("confirmed"), None).await
    }

    pub async fn list_alerts(&self, unread_only: bool) -> Result<Vec<AlertRow>, sqlx::Error> {
        let sql = if unread_only {
            r#"
            SELECT id, pattern_id, alert_type::text, title, body, read_at, created_at
            FROM subscription_alerts
            WHERE read_at IS NULL
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, pattern_id, alert_type::text, title, body, read_at, created_at
            FROM subscription_alerts
            ORDER BY created_at DESC
            LIMIT 100
            "#
        };
        sqlx::query_as::<_, AlertRow>(sql)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn mark_alert_read(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE subscription_alerts SET read_at = NOW() WHERE id = $1 AND read_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn recent_price_events(&self, limit: i64) -> Result<Vec<PriceEventRow>, sqlx::Error> {
        sqlx::query_as::<_, PriceEventRow>(
            r#"
            SELECT id, event_type, amount::float8 AS amount, previous_amount::float8 AS previous_amount,
                   delta_pct::float8 AS delta_pct, interval_days, occurred_at
            FROM subscription_price_events
            ORDER BY occurred_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn price_history(&self, pattern_id: Uuid) -> Result<Vec<PriceEventRow>, sqlx::Error> {
        sqlx::query_as::<_, PriceEventRow>(
            r#"
            SELECT id, event_type, amount::float8 AS amount, previous_amount::float8 AS previous_amount,
                   delta_pct::float8 AS delta_pct, interval_days, occurred_at
            FROM subscription_price_events
            WHERE pattern_id = $1
            ORDER BY occurred_at ASC
            "#,
        )
        .bind(pattern_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_confirmed_amount(
        &self,
        id: Uuid,
        amount: f64,
        interval_days: i32,
        last_seen: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE subscription_patterns
            SET current_amount = $2, interval_days = $3, last_seen_at = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(amount)
        .bind(interval_days)
        .bind(last_seen)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TransactionDbRow {
    firefly_id: String,
    account_id: Option<String>,
    date: NaiveDate,
    amount: f64,
    description: Option<String>,
    category_id: Option<String>,
    payload: serde_json::Value,
}

impl From<TransactionDbRow> for TransactionRow {
    fn from(r: TransactionDbRow) -> Self {
        Self {
            firefly_id: r.firefly_id,
            account_id: r.account_id,
            date: r.date,
            amount: r.amount,
            description: r.description,
            category_id: r.category_id,
            payload: r.payload,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ConfirmedDbRow {
    payee_key: String,
    amount: f64,
    interval_days: i32,
    fingerprint: String,
}
