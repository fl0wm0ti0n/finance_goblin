use std::collections::HashSet;

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::config::SubscriptionsConfig;
use crate::forecast::types::TransactionRow;
use crate::recurrence::{RecurrenceGroup, normalize};

use super::tags::{normalize_slug, validate_tag_name};
use super::types::{
    AlertRow, ConfirmFromDiscoverError, ConfirmFromDiscoverResult, ConfirmedPayeeInterval,
    OperatorTagRow, OperatorTagSummary, PatternDetailRow, PatternRow, PendingUpsertOutcome,
    PriceEventRow, UnreadAlertCountResponse,
};

/// DEC-0086 — ±3 day tolerance for monthly cadence variance.
pub fn interval_matches(stored: i32, detected: i32) -> bool {
    (stored - detected).abs() <= 3
}

pub fn find_confirmed_payee_interval<'a>(
    confirmed: &'a [ConfirmedPayeeInterval],
    payee_key: &str,
    interval_days: i32,
) -> Option<&'a ConfirmedPayeeInterval> {
    confirmed
        .iter()
        .find(|row| row.payee_key == payee_key && interval_matches(row.interval_days, interval_days))
}

pub fn is_rejected_payee_interval(
    rejected: &[(String, i32)],
    payee_key: &str,
    interval_days: i32,
) -> bool {
    rejected
        .iter()
        .any(|(pk, iv)| pk == payee_key && interval_matches(*iv, interval_days))
}

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
        account_id: Option<&str>,
    ) -> Result<Vec<TransactionRow>, sqlx::Error> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(window_days);
        let rows = if let Some(account_id) = account_id {
            sqlx::query_as::<_, TransactionDbRow>(
                r#"
                SELECT firefly_id, account_id, date, amount::float8 AS amount, description, category_id, payload
                FROM transactions
                WHERE date >= $1 AND amount < 0 AND account_id = $2
                ORDER BY date ASC
                "#,
            )
            .bind(cutoff)
            .bind(account_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, TransactionDbRow>(
                r#"
                SELECT firefly_id, account_id, date, amount::float8 AS amount, description, category_id, payload
                FROM transactions
                WHERE date >= $1 AND amount < 0
                ORDER BY date ASC
                "#,
            )
            .bind(cutoff)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn load_transactions_by_ids(
        &self,
        transaction_ids: &[String],
    ) -> Result<Vec<TransactionRow>, sqlx::Error> {
        if transaction_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query_as::<_, TransactionDbRow>(
            r#"
            SELECT firefly_id, account_id, date, amount::float8 AS amount, description, category_id, payload
            FROM transactions
            WHERE firefly_id = ANY($1)
            ORDER BY date ASC
            "#,
        )
        .bind(transaction_ids)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn compute_display_category_id(
        &self,
        pattern_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let category_id: Option<String> = sqlx::query_scalar(
            r#"
            WITH linked AS (
              SELECT t.category_id, t.date
              FROM subscription_pattern_transactions spt
              JOIN transactions t ON t.firefly_id = spt.transaction_firefly_id
              WHERE spt.pattern_id = $1 AND t.category_id IS NOT NULL
            ),
            ranked AS (
              SELECT category_id,
                     COUNT(*) AS cnt,
                     MAX(date) AS last_date,
                     RANK() OVER (ORDER BY COUNT(*) DESC, MAX(date) DESC) AS rnk
              FROM linked
              GROUP BY category_id
            )
            SELECT category_id FROM ranked WHERE rnk = 1 LIMIT 1
            "#,
        )
        .bind(pattern_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(category_id)
    }

    pub async fn refresh_display_category_id(&self, pattern_id: Uuid) -> Result<(), sqlx::Error> {
        let category_id = self.compute_display_category_id(pattern_id).await?;
        sqlx::query(
            "UPDATE subscription_patterns SET display_category_id = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(pattern_id)
        .bind(category_id)
        .execute(&self.pool)
        .await?;
        Ok(())
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

    pub async fn load_confirmed_payee_intervals(
        &self,
    ) -> Result<Vec<ConfirmedPayeeInterval>, sqlx::Error> {
        sqlx::query_as::<_, ConfirmedPayeeInterval>(
            r#"
            SELECT id, fingerprint, payee_key, interval_days, confirmed_at
            FROM subscription_patterns
            WHERE status = 'confirmed'
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn load_rejected_payee_intervals(&self) -> Result<Vec<(String, i32)>, sqlx::Error> {
        let rows: Vec<(String, i32)> = sqlx::query_as(
            r#"
            SELECT payee_key, interval_days
            FROM subscription_patterns
            WHERE status = 'rejected'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// DEC-0085/0086 — refresh confirmed row in-place; rotate fingerprint; preserve confirmed_at.
    /// Returns `Ok(false)` on UNIQUE fingerprint conflict (fail-safe to pending path).
    pub async fn merge_confirmed_pattern(
        &self,
        existing_id: Uuid,
        group: &RecurrenceGroup,
        new_fingerprint: &str,
        kind: &str,
        sync_run_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let first = group
            .transaction_dates
            .first()
            .copied()
            .unwrap_or_else(|| Utc::now().date_naive());
        let last = group
            .transaction_dates
            .last()
            .copied()
            .unwrap_or(first);

        let mut tx = self.pool.begin().await?;
        let update = sqlx::query(
            r#"
            UPDATE subscription_patterns
            SET fingerprint = $2,
                kind = $3::subscription_kind,
                payee_key = $4,
                display_name = $5,
                interval_days = $6,
                current_amount = $7,
                confidence_pct = GREATEST(confidence_pct, $8),
                last_seen_at = GREATEST(last_seen_at, $9),
                detection_run_id = $10,
                updated_at = NOW()
            WHERE id = $1 AND status = 'confirmed'
            "#,
        )
        .bind(existing_id)
        .bind(new_fingerprint)
        .bind(kind)
        .bind(&group.payee_key)
        .bind(&group.display_name)
        .bind(group.interval_days as i32)
        .bind(group.median_amount)
        .bind(group.confidence_pct)
        .bind(last)
        .bind(sync_run_id)
        .execute(&mut *tx)
        .await;

        match update {
            Ok(result) if result.rows_affected() == 0 => {
                tx.rollback().await?;
                return Ok(false);
            }
            Ok(_) => {}
            Err(e) if is_unique_violation(&e) => {
                warn!(
                    %existing_id,
                    %new_fingerprint,
                    "merge_confirmed_pattern fingerprint conflict — fail-safe to pending"
                );
                tx.rollback().await?;
                return Ok(false);
            }
            Err(e) => return Err(e),
        }

        for tx_id in &group.transaction_ids {
            sqlx::query(
                r#"
                INSERT INTO subscription_pattern_transactions (pattern_id, transaction_firefly_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(existing_id)
            .bind(tx_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.refresh_display_category_id(existing_id).await?;
        Ok(true)
    }

    pub async fn confirm_from_discover(
        &self,
        payee_key: &str,
        interval_days: i32,
        median_amount: f64,
        transaction_ids: &[String],
        kind: &str,
        sync_run_id: Uuid,
    ) -> Result<Result<ConfirmFromDiscoverResult, ConfirmFromDiscoverError>, sqlx::Error> {
        let txs = self.load_transactions_by_ids(transaction_ids).await?;
        if txs.len() != transaction_ids.len() {
            return Ok(Err(ConfirmFromDiscoverError::InvalidTransactions(
                "one or more transaction ids not found".into(),
            )));
        }

        for tx in &txs {
            let desc_key = normalize::payee_key(tx.description.as_deref().unwrap_or(""));
            if desc_key != payee_key {
                return Ok(Err(ConfirmFromDiscoverError::InvalidTransactions(
                    "transactions do not share normalized payee_key".into(),
                )));
            }
        }

        let mut dates: Vec<NaiveDate> = txs.iter().map(|t| t.date).collect();
        dates.sort();
        let display_name = txs
            .last()
            .and_then(|t| t.description.clone())
            .unwrap_or_else(|| payee_key.to_string());

        let group = RecurrenceGroup {
            payee_key: payee_key.to_string(),
            display_name,
            interval_days: interval_days as i64,
            median_amount,
            confidence_pct: 95,
            transaction_ids: transaction_ids.to_vec(),
            transaction_dates: dates.clone(),
            category_ids: txs.iter().map(|t| t.category_id.clone()).collect(),
        };

        let fingerprint =
            crate::recurrence::compute_fingerprint(payee_key, interval_days as i64, median_amount);

        let confirmed_payee_intervals = self.load_confirmed_payee_intervals().await?;
        let rejected_payee_intervals = self.load_rejected_payee_intervals().await?;

        if is_rejected_payee_interval(&rejected_payee_intervals, payee_key, interval_days) {
            return Ok(Err(ConfirmFromDiscoverError::RejectedPayeeInterval));
        }

        if let Some(confirmed) =
            find_confirmed_payee_interval(&confirmed_payee_intervals, payee_key, interval_days)
        {
            let merged = self
                .merge_confirmed_pattern(confirmed.id, &group, &fingerprint, kind, sync_run_id)
                .await?;
            if !merged {
                return Ok(Err(ConfirmFromDiscoverError::FingerprintConflict));
            }
            let pattern = self
                .get_pattern_row(confirmed.id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound)?;
            return Ok(Ok(ConfirmFromDiscoverResult {
                pattern,
                merged: true,
            }));
        }

        let first = dates.first().copied().unwrap_or_else(|| Utc::now().date_naive());
        let last = dates.last().copied().unwrap_or(first);

        let mut tx = self.pool.begin().await?;
        let pattern_id: Uuid = match sqlx::query_scalar(
            r#"
            INSERT INTO subscription_patterns (
                fingerprint, status, kind, payee_key, display_name, interval_days,
                current_amount, confidence_pct, first_seen_at, last_seen_at,
                confirmed_at, detection_run_id
            )
            VALUES ($1, 'confirmed', $2::subscription_kind, $3, $4, $5, $6, $7, $8, $9, NOW(), $10)
            RETURNING id
            "#,
        )
        .bind(&fingerprint)
        .bind(kind)
        .bind(payee_key)
        .bind(&group.display_name)
        .bind(interval_days)
        .bind(median_amount)
        .bind(group.confidence_pct)
        .bind(first)
        .bind(last)
        .bind(sync_run_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(id) => id,
            Err(e) if is_unique_violation(&e) => {
                tx.rollback().await?;
                return Ok(Err(ConfirmFromDiscoverError::FingerprintConflict));
            }
            Err(e) => return Err(e),
        };

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
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.refresh_display_category_id(pattern_id).await?;

        let pattern = self
            .get_pattern_row(pattern_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;
        Ok(Ok(ConfirmFromDiscoverResult {
            pattern,
            merged: false,
        }))
    }

    async fn get_pattern_row(&self, id: Uuid) -> Result<Option<PatternRow>, sqlx::Error> {
        sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, fingerprint, status::text, kind::text, payee_key, display_name,
                   interval_days, current_amount::float8 AS current_amount, confidence_pct,
                   first_seen_at, last_seen_at, confirmed_at, rejected_at, display_category_id,
                   created_at, updated_at
            FROM subscription_patterns
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
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
    ) -> Result<PendingUpsertOutcome, sqlx::Error> {
        let prior: Option<(String, i16)> = sqlx::query_as(
            "SELECT status::text, confidence_pct FROM subscription_patterns WHERE fingerprint = $1",
        )
        .bind(fingerprint)
        .fetch_optional(&self.pool)
        .await?;

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

        let emit_detection_alert = match prior {
            None => true,
            Some((status, old_conf)) if status == "pending" => group.confidence_pct > old_conf,
            _ => false,
        };

        Ok(PendingUpsertOutcome {
            id,
            emit_detection_alert,
        })
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

    pub fn compute_alert_fingerprint(
        alert_type: &str,
        pattern_id: Uuid,
        direction: Option<&str>,
        amount: Option<f64>,
        interval_days: Option<i32>,
    ) -> String {
        match alert_type {
            "new_detection" => format!("sub_alert:new_detection:{pattern_id}"),
            "price_change" => {
                let dir = direction.unwrap_or("unknown");
                let amt = amount.map(|a| (a * 100.0).round() / 100.0).unwrap_or(0.0);
                format!("sub_alert:price_change:{pattern_id}:{dir}:{amt:.2}")
            }
            "interval_change" => {
                let days = interval_days.unwrap_or(0);
                format!("sub_alert:interval_change:{pattern_id}:{days}")
            }
            _ => format!("sub_alert:unknown:{pattern_id}"),
        }
    }

    pub async fn upsert_alert(
        &self,
        pattern_id: Option<Uuid>,
        alert_type: &str,
        title: &str,
        body: Option<&str>,
        sync_run_id: Uuid,
        fingerprint: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO subscription_alerts (pattern_id, alert_type, title, body, sync_run_id, fingerprint)
            VALUES ($1, $2::subscription_alert_type, $3, $4, $5, $6)
            ON CONFLICT (fingerprint) WHERE read_at IS NULL
            DO UPDATE SET
                body = EXCLUDED.body,
                sync_run_id = EXCLUDED.sync_run_id,
                created_at = NOW()
            "#,
        )
        .bind(pattern_id)
        .bind(alert_type)
        .bind(title)
        .bind(body)
        .bind(sync_run_id)
        .bind(fingerprint)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn unread_alert_counts(&self) -> Result<UnreadAlertCountResponse, sqlx::Error> {
        let unread_total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE read_at IS NULL",
        )
        .fetch_one(&self.pool)
        .await?;

        let unread_new_detection: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint FROM subscription_alerts
            WHERE read_at IS NULL AND alert_type = 'new_detection'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let unread_price_change: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint FROM subscription_alerts
            WHERE read_at IS NULL AND alert_type = 'price_change'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let pending_patterns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'pending'",
        )
        .fetch_one(&self.pool)
        .await?;

        let orphan_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM subscription_alerts sa
            LEFT JOIN subscription_patterns sp
                ON sp.id = sa.pattern_id AND sp.status = 'pending'
            WHERE sa.read_at IS NULL
              AND sa.alert_type = 'new_detection'
              AND sp.id IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let reconciled = unread_new_detection <= pending_patterns && orphan_count == 0;
        let reconciliation_note = "unread_new_detection counts pending patterns with unread new_detection alerts; price_change alerts are informational".to_string();

        Ok(UnreadAlertCountResponse {
            unread_total,
            unread_new_detection,
            unread_price_change,
            pending_patterns,
            reconciled,
            reconciliation_note,
        })
    }

    pub async fn mark_read_unread_alerts_for_pattern(
        &self,
        pattern_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE subscription_alerts SET read_at = NOW() WHERE pattern_id = $1 AND read_at IS NULL",
        )
        .bind(pattern_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_patterns(
        &self,
        status: Option<&str>,
        kind: Option<&str>,
        tag_slug: Option<&str>,
    ) -> Result<Vec<PatternRow>, sqlx::Error> {
        let mut sql = String::from(
            r#"
            SELECT p.id, p.fingerprint, p.status::text, p.kind::text, p.payee_key, p.display_name,
                   p.interval_days, p.current_amount::float8 AS current_amount, p.confidence_pct,
                   p.first_seen_at, p.last_seen_at, p.confirmed_at, p.rejected_at, p.display_category_id,
                   p.created_at, p.updated_at
            FROM subscription_patterns p
            "#,
        );
        if tag_slug.is_some() {
            sql.push_str(
                r#"
                INNER JOIN subscription_pattern_tags spt ON spt.pattern_id = p.id
                INNER JOIN operator_tags ot ON ot.id = spt.tag_id
                "#,
            );
        }
        sql.push_str(" WHERE 1=1");
        let mut bind_idx = 1;
        if tag_slug.is_some() {
            sql.push_str(&format!(" AND ot.slug = ${bind_idx}"));
            bind_idx += 1;
        }
        if status.is_some() {
            sql.push_str(&format!(" AND p.status = ${bind_idx}::subscription_status"));
            bind_idx += 1;
        }
        if kind.is_some() {
            sql.push_str(&format!(" AND p.kind = ${bind_idx}::subscription_kind"));
        }
        sql.push_str(" ORDER BY p.last_seen_at DESC");

        let mut q = sqlx::query_as::<_, PatternRow>(&sql);
        if let Some(slug) = tag_slug {
            q = q.bind(slug);
        }
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(k) = kind {
            q = q.bind(k);
        }
        q.fetch_all(&self.pool).await
    }

    pub async fn list_tags_for_patterns(
        &self,
        pattern_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<OperatorTagSummary>>, sqlx::Error> {
        if pattern_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT spt.pattern_id, ot.id, ot.name, ot.slug
            FROM subscription_pattern_tags spt
            INNER JOIN operator_tags ot ON ot.id = spt.tag_id
            WHERE spt.pattern_id = ANY($1)
            ORDER BY ot.name ASC
            "#,
        )
        .bind(pattern_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut map: std::collections::HashMap<Uuid, Vec<OperatorTagSummary>> =
            std::collections::HashMap::new();
        for (pattern_id, id, name, slug) in rows {
            map.entry(pattern_id).or_default().push(OperatorTagSummary {
                id,
                name,
                slug,
            });
        }
        Ok(map)
    }

    pub async fn get_pattern(&self, id: Uuid) -> Result<Option<PatternDetailRow>, sqlx::Error> {
        sqlx::query_as::<_, PatternDetailRow>(
            r#"
            SELECT p.id, p.fingerprint, p.status::text, p.kind::text, p.payee_key, p.display_name,
                   p.interval_days, p.current_amount::float8 AS current_amount, p.confidence_pct,
                   p.first_seen_at, p.last_seen_at, p.confirmed_at, p.rejected_at, p.display_category_id,
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
                      interval_days, current_amount::float8 AS current_amount, confidence_pct,
                      first_seen_at, last_seen_at, confirmed_at, rejected_at, display_category_id,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(kind)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref pattern) = row {
            self.mark_read_unread_alerts_for_pattern(pattern.id).await?;
            self.refresh_display_category_id(pattern.id).await?;
            if let Some(updated) = self.get_pattern_row(pattern.id).await? {
                return Ok(Some(updated));
            }
        }

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
                      interval_days, current_amount::float8 AS current_amount, confidence_pct,
                      first_seen_at, last_seen_at, confirmed_at, rejected_at, display_category_id,
                      created_at, updated_at
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

        if let Some(ref pattern) = row {
            self.mark_read_unread_alerts_for_pattern(pattern.id).await?;
        }

        Ok(row)
    }

    pub async fn mark_inactive(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE subscription_patterns SET status = 'inactive', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.mark_read_unread_alerts_for_pattern(id).await?;
        Ok(())
    }

    pub async fn list_confirmed_patterns(&self) -> Result<Vec<PatternRow>, sqlx::Error> {
        self.list_patterns(Some("confirmed"), None, None).await
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

    pub async fn list_operator_tags(&self) -> Result<Vec<OperatorTagRow>, sqlx::Error> {
        sqlx::query_as::<_, OperatorTagRow>(
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM operator_tags
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_operator_tag(&self, name: &str) -> Result<OperatorTagRow, sqlx::Error> {
        validate_tag_name(name).map_err(|_| sqlx::Error::RowNotFound)?;
        let slug = normalize_slug(name);
        sqlx::query_as::<_, OperatorTagRow>(
            r#"
            INSERT INTO operator_tags (name, slug)
            VALUES ($1, $2)
            RETURNING id, name, slug, created_at, updated_at
            "#,
        )
        .bind(name.trim())
        .bind(slug)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn rename_operator_tag(
        &self,
        id: Uuid,
        name: &str,
    ) -> Result<Option<OperatorTagRow>, sqlx::Error> {
        validate_tag_name(name).map_err(|_| sqlx::Error::RowNotFound)?;
        let slug = normalize_slug(name);
        sqlx::query_as::<_, OperatorTagRow>(
            r#"
            UPDATE operator_tags
            SET name = $2, slug = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, slug, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name.trim())
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_operator_tag(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM operator_tags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn operator_tag_exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM operator_tags WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count > 0)
    }

    pub async fn replace_pattern_tags(
        &self,
        pattern_id: Uuid,
        tag_ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        for tag_id in tag_ids {
            if !self.operator_tag_exists(*tag_id).await? {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM subscription_pattern_tags WHERE pattern_id = $1")
            .bind(pattern_id)
            .execute(&mut *tx)
            .await?;

        for tag_id in tag_ids {
            sqlx::query(
                r#"
                INSERT INTO subscription_pattern_tags (pattern_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(pattern_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
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

/// DEC-0100 — deterministic majority category from linked (category_id, date) pairs.
pub fn majority_category_id(category_dates: &[(Option<String>, NaiveDate)]) -> Option<String> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, (i64, NaiveDate)> = HashMap::new();
    for (cat, date) in category_dates {
        let Some(id) = cat else { continue };
        let entry = counts.entry(id.clone()).or_insert((0, *date));
        entry.0 += 1;
        if *date > entry.1 {
            entry.1 = *date;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| {
            a.1 .0
                .cmp(&b.1 .0)
                .then_with(|| a.1 .1.cmp(&b.1 .1))
        })
        .map(|(id, _)| id)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db| db.code())
        .map(|code| code == "23505")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_matches_within_three_days() {
        assert!(interval_matches(30, 30));
        assert!(interval_matches(30, 28));
        assert!(interval_matches(30, 33));
        assert!(interval_matches(30, 27));
        assert!(!interval_matches(30, 26));
        assert!(!interval_matches(30, 34));
    }

    #[test]
    fn find_confirmed_payee_interval_uses_tolerance() {
        let rows = vec![ConfirmedPayeeInterval {
            id: Uuid::new_v4(),
            fingerprint: "fp".into(),
            payee_key: "cursor".into(),
            interval_days: 30,
            confirmed_at: None,
        }];
        assert!(find_confirmed_payee_interval(&rows, "cursor", 31).is_some());
        assert!(find_confirmed_payee_interval(&rows, "cursor", 26).is_none());
        assert!(find_confirmed_payee_interval(&rows, "apple", 30).is_none());
    }

    #[test]
    fn is_rejected_payee_interval_uses_tolerance() {
        let rejected = vec![("apple".to_string(), 30)];
        assert!(is_rejected_payee_interval(&rejected, "apple", 32));
        assert!(!is_rejected_payee_interval(&rejected, "apple", 25));
    }

    #[test]
    fn majority_category_picks_mode_then_latest_date() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let d3 = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let rows = vec![
            (Some("cat-a".into()), d1),
            (Some("cat-a".into()), d2),
            (Some("cat-b".into()), d3),
            (None, d3),
        ];
        assert_eq!(majority_category_id(&rows).as_deref(), Some("cat-a"));
    }

    #[test]
    fn majority_category_tie_breaks_by_latest_date() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let rows = vec![
            (Some("cat-a".into()), d1),
            (Some("cat-b".into()), d2),
        ];
        assert_eq!(majority_category_id(&rows).as_deref(), Some("cat-b"));
    }

    #[test]
    fn majority_category_all_uncategorized_returns_none() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(majority_category_id(&[(None, d1)]), None);
    }
}
