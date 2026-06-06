use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/portfolio/pnl", get(pnl_history))
}

#[derive(Deserialize)]
struct DaysQuery {
    #[serde(default = "default_days")]
    days: u32,
}

fn default_days() -> u32 {
    90
}

async fn pnl_history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DaysQuery>,
) -> Result<Json<Vec<crate::portfolio::repository::PnlSnapshotRow>>, StatusCode> {
    state
        .portfolio
        .repository()
        .fetch_pnl_history(q.days)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
