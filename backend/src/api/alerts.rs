use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::alerts::{AlertError, AlertListFilter};
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/alerts", get(list_alerts))
        .route("/api/v1/alerts/unread-count", get(unread_count))
        .route("/api/v1/alerts/{id}/acknowledge", patch(acknowledge))
        .route("/api/v1/alerts/{id}/dismiss", patch(dismiss))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    #[serde(default)]
    include_dismissed: bool,
}

async fn list_alerts(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<crate::alerts::types::AlertRow>>, StatusCode> {
    let filter = AlertListFilter {
        status: q.status.or_else(|| Some("active".into())),
        include_dismissed: q.include_dismissed,
        limit: Some(100),
    };
    state
        .alerts
        .list(filter)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(serde::Serialize)]
struct UnreadCountResponse {
    count: u32,
}

async fn unread_count(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let count = state
        .alerts
        .unread_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(UnreadCountResponse { count }))
}

async fn acknowledge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::alerts::types::AlertRow>, StatusCode> {
    match state.alerts.acknowledge(id).await {
        Ok(row) => Ok(Json(row)),
        Err(AlertError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn dismiss(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::alerts::types::AlertRow>, StatusCode> {
    match state.alerts.dismiss(id).await {
        Ok(row) => Ok(Json(row)),
        Err(AlertError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
