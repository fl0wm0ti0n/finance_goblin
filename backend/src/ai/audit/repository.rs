use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct AuditRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_subject: String,
    pub tool_name: String,
    pub args_summary: Value,
    pub result_status: String,
    pub result_rows: Option<i32>,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AuditInsert {
    pub session_id: Uuid,
    pub user_subject: String,
    pub tool_name: String,
    pub args_summary: Value,
    pub result_status: String,
    pub result_rows: Option<i32>,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, row: AuditInsert) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ai_tool_audit (
                id, session_id, user_subject, tool_name, args_summary,
                result_status, result_rows, duration_ms, error_message, model, provider
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(id)
        .bind(row.session_id)
        .bind(row.user_subject)
        .bind(row.tool_name)
        .bind(row.args_summary)
        .bind(row.result_status)
        .bind(row.result_rows)
        .bind(row.duration_ms)
        .bind(row.error_message)
        .bind(row.model)
        .bind(row.provider)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<AuditRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, session_id, user_subject, tool_name, args_summary,
                   result_status, result_rows, duration_ms, error_message, model, provider, created_at
            FROM ai_tool_audit
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn purge_expired(&self, retention_days: u32, max_rows: u32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM ai_tool_audit WHERE created_at < NOW() - ($1::int * INTERVAL '1 day')",
        )
        .bind(retention_days as i32)
        .execute(&self.pool)
        .await?;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM ai_tool_audit")
            .fetch_one(&self.pool)
            .await?;

        if count > max_rows as i64 {
            let excess = count - max_rows as i64;
            sqlx::query(
                r#"
                DELETE FROM ai_tool_audit
                WHERE id IN (
                    SELECT id FROM ai_tool_audit
                    ORDER BY created_at ASC
                    LIMIT $1
                )
                "#,
            )
            .bind(excess)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
