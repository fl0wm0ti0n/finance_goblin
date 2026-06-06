use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
struct AuditItem {
    id: String,
    session_id: String,
    user_subject: String,
    tool_name: String,
    args_summary: serde_json::Value,
    result_status: String,
    duration_ms: i32,
    error_message: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    created_at: String,
}

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/api/v1/ai/audit", axum::routing::get(list_audit))
}

pub async fn list_audit(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditItem>>, StatusCode> {
    let rows = state
        .ai
        .list_audit(q.limit.min(200), q.offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|r| AuditItem {
                id: r.id.to_string(),
                session_id: r.session_id.to_string(),
                user_subject: r.user_subject,
                tool_name: r.tool_name,
                args_summary: r.args_summary,
                result_status: r.result_status,
                duration_ms: r.duration_ms,
                error_message: r.error_message,
                model: r.model,
                provider: r.provider,
                created_at: r.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}
