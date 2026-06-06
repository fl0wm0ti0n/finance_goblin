use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ReadyResponse {
    status: &'static str,
    database: &'static str,
    firefly_pat_configured: bool,
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(liveness))
        .route("/health/ready", get(readiness))
        .with_state(state)
}

async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn readiness(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ReadyResponse>) {
    let firefly_pat_configured = state.config.firefly.pat_configured();
    if state.db.is_ready().await {
        (
            StatusCode::OK,
            Json(ReadyResponse {
                status: "ready",
                database: "connected",
                firefly_pat_configured,
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "not_ready",
                database: "unreachable",
                firefly_pat_configured,
            }),
        )
    }
}
