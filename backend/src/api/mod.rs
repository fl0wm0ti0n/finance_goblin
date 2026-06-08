use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::require_auth;
use crate::sync::TriggerError;
use crate::AppState;

mod ai_audit;
mod ai_test;
mod alerts;
mod categories;
mod chat;
mod exchanges;
mod forecast;
mod plans;
mod portfolio;
mod subscription_tags;
mod subscriptions;
mod wealth;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/sync/status", get(sync_status))
        .route("/api/v1/sync/runs", get(sync_runs))
        .route("/api/v1/sync/entities", get(sync_entities))
        .route("/api/v1/sync/trigger", post(sync_trigger))
        .route("/api/v1/settings", get(settings))
        .route("/api/v1/forecast/meta", get(forecast::meta))
        .route("/api/v1/forecast/accounts", get(forecast::accounts))
        .route("/api/v1/forecast/daily", get(forecast::daily))
        .route("/api/v1/forecast/monthly", get(forecast::monthly))
        .route("/api/v1/forecast/long-term", get(forecast::long_term))
        .route("/api/v1/forecast/compare", get(forecast::compare))
        .route("/api/v1/forecast/aggregate", get(forecast::aggregate))
        .merge(categories::routes())
        .merge(plans::routes())
        .merge(subscriptions::routes())
        .merge(subscription_tags::routes())
        .merge(exchanges::routes())
        .merge(wealth::routes())
        .merge(portfolio::routes())
        .merge(alerts::routes())
        .merge(chat::routes())
        .merge(ai_test::routes())
        .merge(ai_audit::routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(middleware::from_fn_with_state(state.clone(), inject_auth_user))
        .with_state(state)
}

async fn sync_status(State(state): State<Arc<AppState>>) -> Json<crate::sync::SyncStatusResponse> {
    Json(state.sync.status().await)
}

async fn sync_runs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::sync::SyncRunRow>>, StatusCode> {
    state
        .sync
        .list_runs(50)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn sync_entities(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::sync::EntityCounts>, StatusCode> {
    state
        .sync
        .entity_counts()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize)]
struct TriggerResponse {
    run_id: Uuid,
    status: &'static str,
}

async fn sync_trigger(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<TriggerResponse>), (StatusCode, Json<serde_json::Value>)> {
    match state.sync.trigger_manual().await {
        Ok(run_id) => Ok((
            StatusCode::ACCEPTED,
            Json(TriggerResponse {
                run_id,
                status: "accepted",
            }),
        )),
        Err(TriggerError::AlreadyRunning { run_id }) => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "sync_already_running",
                "active_run_id": run_id
            })),
        )),
    }
}

#[derive(Serialize)]
struct SettingsResponse {
    firefly_base_url: String,
    firefly_auth_method: &'static str,
    database_mode: String,
    database_host: String,
    sync_interval_seconds: u64,
    oidc_issuer_url: String,
    read_only: bool,
    ai: crate::config::AiPublicSettings,
    privacy: crate::config::PrivacyConfig,
    provider_configured: bool,
    openai_configured: bool,
    exchanges: crate::config::ExchangesSettingsView,
    portfolio: crate::config::PortfolioSettingsView,
}

async fn settings(State(state): State<Arc<AppState>>) -> Json<SettingsResponse> {
    Json(SettingsResponse {
        firefly_base_url: state.config.firefly.base_url.clone(),
        firefly_auth_method: "personal_access_token",
        database_mode: state.config.database.mode.clone(),
        database_host: state.config.database.host.clone(),
        sync_interval_seconds: state.config.sync.interval_seconds,
        oidc_issuer_url: state.config.oidc.issuer_url.clone(),
        read_only: true,
        ai: state.ai.ai_public_settings(),
        privacy: state.config.privacy.clone(),
        provider_configured: state.config.ai.provider_configured(),
        openai_configured: state.config.ai.provider_configured(),
        exchanges: state.config.exchanges.settings_view(),
        portfolio: state.config.portfolio.settings_view(),
    })
}

async fn inject_auth_user(
    State(state): State<Arc<AppState>>,
    mut req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let subject = if state.config.oidc.dev_bypass {
        "dev-bypass".to_string()
    } else {
        req.extensions()
            .get::<crate::auth::AuthUser>()
            .map(|u| u.subject.clone())
            .unwrap_or_else(|| "authenticated".into())
    };
    req.headers_mut().insert(
        "x-auth-user",
        axum::http::HeaderValue::from_str(&subject).unwrap_or_else(|_| {
            axum::http::HeaderValue::from_static("authenticated")
        }),
    );
    next.run(req).await
}
