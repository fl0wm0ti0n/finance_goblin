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
    Router::new()
        .route("/api/v1/wealth", get(breakdown))
        .route("/api/v1/wealth/crypto", get(crypto))
        .route("/api/v1/wealth/history", get(history))
        .route("/api/v1/wealth/portfolio-forecast", get(portfolio_forecast))
}

async fn breakdown(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::wealth::types::ExtendedWealthBreakdown>, StatusCode> {
    let allocation = load_allocation_target(&state).await;
    state
        .wealth
        .compute_extended(
            Some(state.exchanges.repository().as_ref()),
            Some(&state.portfolio),
            allocation,
        )
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn crypto(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::wealth::types::CryptoBreakdown>, StatusCode> {
    let extended = state
        .wealth
        .compute_extended(
            Some(state.exchanges.repository().as_ref()),
            Some(&state.portfolio),
            None,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(extended.crypto))
}

#[derive(Deserialize)]
struct HistoryQuery {
    #[serde(default = "default_days")]
    days: u32,
}

fn default_days() -> u32 {
    90
}

async fn history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<crate::wealth::types::WealthHistoryPoint>>, StatusCode> {
    state
        .wealth
        .history(q.days)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn portfolio_forecast(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::wealth::portfolio_forecast::PortfolioForecastResponse>, StatusCode> {
    let extended = state
        .wealth
        .compute_extended(
            Some(state.exchanges.repository().as_ref()),
            Some(&state.portfolio),
            None,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state
        .portfolio_forecast
        .latest(extended.fx_incomplete)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn load_allocation_target(
    state: &AppState,
) -> Option<crate::wealth::types::AllocationWeights> {
    let active = state.plans.active_plan().await.ok()??;
    let adjustments = state.plans.load_adjustments(active.latest_version_id).await.ok()?;
    for adj in adjustments {
        if adj.target_type == crate::plan::types::AdjustmentTarget::AllocationTarget {
            if let Some(ref label) = adj.label {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(label) {
                    return Some(crate::wealth::types::AllocationWeights {
                        etf_traditional_pct: v["weights"]["etf_traditional_pct"]
                            .as_f64()
                            .unwrap_or(0.0),
                        crypto_pct: v["weights"]["crypto_pct"].as_f64().unwrap_or(0.0),
                        cash_pct: v["weights"]["cash_pct"].as_f64().unwrap_or(0.0),
                    });
                }
            }
        }
    }
    None
}
