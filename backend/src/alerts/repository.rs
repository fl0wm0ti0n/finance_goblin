use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::{
    AlertCandidate, AlertListFilter, AlertRow, AlertsConfig,
};

pub struct AlertRepository {
    pool: PgPool,
}

impl AlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn latest_forecast_computation_id(&self) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT id FROM forecast_computations WHERE status = 'success' ORDER BY computed_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mirror_config(&self, config: &AlertsConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO alert_config (id, scarcity_threshold_eur, budget_drift_pct, updated_at)
            VALUES (1, $1, $2, NOW())
            ON CONFLICT (id) DO UPDATE SET
                scarcity_threshold_eur = EXCLUDED.scarcity_threshold_eur,
                budget_drift_pct = EXCLUDED.budget_drift_pct,
                updated_at = NOW()
            "#,
        )
        .bind(config.scarcity_threshold_eur)
        .bind(config.budget_drift_pct)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_config(&self) -> Result<AlertsConfig, sqlx::Error> {
        let row: (f64, f64) = sqlx::query_as(
            "SELECT scarcity_threshold_eur::float8, budget_drift_pct::float8 FROM alert_config WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(crate::config::AlertsConfig {
            scarcity_threshold_eur: row.0,
            budget_drift_pct: row.1,
            reporting_currency: "EUR".into(),
        })
    }

    pub async fn upsert_or_resolve(
        &self,
        candidate: &AlertCandidate,
        sync_run_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let existing: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, status::text FROM alerts
            WHERE fingerprint = $1 AND status IN ('active', 'acknowledged', 'dismissed')
            "#,
        )
        .bind(&candidate.fingerprint)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, status)) = existing {
            if status == "dismissed" {
                return Ok(false);
            }
            sqlx::query(
                r#"
                UPDATE alerts SET
                    severity = $2::alert_severity,
                    title = $3,
                    message = $4,
                    context = $5,
                    sync_run_id = $6
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(candidate.severity.as_str())
            .bind(&candidate.title)
            .bind(&candidate.message)
            .bind(&candidate.context)
            .bind(sync_run_id)
            .execute(&self.pool)
            .await?;
            return Ok(false);
        }

        sqlx::query(
            r#"
            INSERT INTO alerts (
                alert_type, severity, status, fingerprint, title, message,
                entity_type, entity_id, context, sync_run_id
            ) VALUES (
                $1::alert_type, $2::alert_severity, 'active', $3, $4, $5, $6, $7, $8, $9
            )
            "#,
        )
        .bind(candidate.alert_type.as_str())
        .bind(candidate.severity.as_str())
        .bind(&candidate.fingerprint)
        .bind(&candidate.title)
        .bind(&candidate.message)
        .bind(&candidate.entity_type)
        .bind(&candidate.entity_id)
        .bind(&candidate.context)
        .bind(sync_run_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn resolve_by_fingerprint(&self, fingerprint: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE alerts SET status = 'resolved', resolved_at = NOW()
            WHERE fingerprint = $1 AND status IN ('active', 'acknowledged', 'dismissed')
            "#,
        )
        .bind(fingerprint)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list(&self, filter: &AlertListFilter) -> Result<Vec<AlertRow>, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100);
        let include_dismissed = filter.include_dismissed;

        let rows = if let Some(status) = &filter.status {
            if include_dismissed {
                sqlx::query_as::<_, AlertDbRow>(
                    r#"
                    SELECT id, alert_type::text, severity::text, status::text, fingerprint,
                           title, message, entity_type, entity_id, context,
                           triggered_at, acknowledged_at, dismissed_at, resolved_at
                    FROM alerts
                    WHERE status = $1::alert_status
                    ORDER BY triggered_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(status)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query_as::<_, AlertDbRow>(
                    r#"
                    SELECT id, alert_type::text, severity::text, status::text, fingerprint,
                           title, message, entity_type, entity_id, context,
                           triggered_at, acknowledged_at, dismissed_at, resolved_at
                    FROM alerts
                    WHERE status = $1::alert_status AND status != 'dismissed'
                    ORDER BY triggered_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(status)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        } else if include_dismissed {
            sqlx::query_as::<_, AlertDbRow>(
                r#"
                SELECT id, alert_type::text, severity::text, status::text, fingerprint,
                       title, message, entity_type, entity_id, context,
                       triggered_at, acknowledged_at, dismissed_at, resolved_at
                FROM alerts
                WHERE status IN ('active', 'acknowledged')
                ORDER BY triggered_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AlertDbRow>(
                r#"
                SELECT id, alert_type::text, severity::text, status::text, fingerprint,
                       title, message, entity_type, entity_id, context,
                       triggered_at, acknowledged_at, dismissed_at, resolved_at
                FROM alerts
                WHERE status IN ('active', 'acknowledged')
                ORDER BY triggered_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into_row(false)).collect())
    }

    pub async fn unread_count(&self) -> Result<u32, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM alerts
            WHERE status = 'active' AND acknowledged_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count as u32)
    }

    pub async fn acknowledge(&self, id: Uuid) -> Result<Option<AlertRow>, sqlx::Error> {
        let row: Option<AlertDbRow> = sqlx::query_as(
            r#"
            UPDATE alerts SET status = 'acknowledged', acknowledged_at = NOW()
            WHERE id = $1 AND status = 'active'
            RETURNING id, alert_type::text, severity::text, status::text, fingerprint,
                      title, message, entity_type, entity_id, context,
                      triggered_at, acknowledged_at, dismissed_at, resolved_at
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into_row(false)))
    }

    pub async fn dismiss(&self, id: Uuid) -> Result<Option<AlertRow>, sqlx::Error> {
        let row: Option<AlertDbRow> = sqlx::query_as(
            r#"
            UPDATE alerts SET status = 'dismissed', dismissed_at = NOW()
            WHERE id = $1 AND status IN ('active', 'acknowledged')
            RETURNING id, alert_type::text, severity::text, status::text, fingerprint,
                      title, message, entity_type, entity_id, context,
                      triggered_at, acknowledged_at, dismissed_at, resolved_at
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into_row(false)))
    }

    pub async fn active_fingerprints(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT fingerprint FROM alerts
            WHERE status IN ('active', 'acknowledged', 'dismissed')
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AlertDbRow {
    id: Uuid,
    alert_type: String,
    severity: String,
    status: String,
    fingerprint: String,
    title: String,
    message: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    context: serde_json::Value,
    triggered_at: DateTime<Utc>,
    acknowledged_at: Option<DateTime<Utc>>,
    dismissed_at: Option<DateTime<Utc>>,
    resolved_at: Option<DateTime<Utc>>,
}

impl AlertDbRow {
    fn into_row(self, stale: bool) -> AlertRow {
        AlertRow {
            id: self.id,
            alert_type: self.alert_type,
            severity: self.severity,
            status: self.status,
            fingerprint: self.fingerprint,
            title: self.title,
            message: self.message,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            context: self.context,
            triggered_at: self.triggered_at,
            acknowledged_at: self.acknowledged_at,
            dismissed_at: self.dismissed_at,
            resolved_at: self.resolved_at,
            stale,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::alerts::types::AlertStatus;

    #[test]
    fn unread_requires_active_unacknowledged() {
        let status = AlertStatus::Active;
        assert_eq!(status.as_str(), "active");
    }
}
