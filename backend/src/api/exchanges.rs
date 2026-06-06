use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/exchanges", get(list_exchanges))
        .route("/api/v1/exchanges/:id/test", post(test_exchange))
        .route("/api/v1/sync/exchanges/trigger", post(trigger_exchanges))
}

async fn list_exchanges(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::exchanges::types::ExchangeListItem>>, StatusCode> {
    state
        .exchanges
        .list_connections()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn test_exchange(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<crate::exchanges::types::ConnectionTest>, StatusCode> {
    state
        .exchanges
        .test_connection(&id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn trigger_exchanges(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.sync.trigger_exchanges_manual().await {
        Ok(run_id) => Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "run_id": run_id, "status": "accepted" })),
        )),
        Err(crate::sync::TriggerError::AlreadyRunning { run_id }) => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "sync_already_running",
                "active_run_id": run_id
            })),
        )),
    }
}
