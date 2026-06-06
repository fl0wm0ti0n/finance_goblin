use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::subscriptions::types::{AlertRow, PatternDetailRow, PatternRow, PriceEventRow};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub kind: Option<String>,
}

#[derive(Deserialize)]
pub struct AlertsQuery {
    pub unread: Option<bool>,
}

#[derive(Deserialize)]
pub struct ConfirmBody {
    pub kind: Option<String>,
}

#[derive(Deserialize)]
pub struct RejectBody {
    pub reason: Option<String>,
}

#[derive(Serialize)]
struct PatternResponse {
    id: String,
    fingerprint: String,
    status: String,
    kind: String,
    payee_key: String,
    display_name: String,
    interval_days: i32,
    current_amount: String,
    confidence_pct: i16,
    first_seen_at: String,
    last_seen_at: String,
    confirmed_at: Option<String>,
    rejected_at: Option<String>,
    transaction_count: Option<i64>,
}

#[derive(Serialize)]
struct PriceHistoryResponse {
    events: Vec<PriceEventItem>,
}

#[derive(Serialize)]
struct PriceEventItem {
    occurred_at: String,
    amount: String,
    event_type: String,
    previous_amount: Option<String>,
    delta_pct: Option<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/subscriptions", get(list_patterns))
        .route("/api/v1/subscriptions/alerts", get(list_alerts))
        .route("/api/v1/subscriptions/alerts/:id/read", patch(mark_alert_read))
        .route("/api/v1/subscriptions/:id", get(get_pattern))
        .route("/api/v1/subscriptions/:id/confirm", post(confirm_pattern))
        .route("/api/v1/subscriptions/:id/reject", post(reject_pattern))
        .route("/api/v1/subscriptions/:id/price-history", get(price_history))
}

fn map_pattern(row: PatternRow) -> PatternResponse {
    let amount = row.amount_f64();
    PatternResponse {
        id: row.id.to_string(),
        fingerprint: row.fingerprint,
        status: row.status,
        kind: row.kind,
        payee_key: row.payee_key,
        display_name: row.display_name,
        interval_days: row.interval_days,
        current_amount: format!("{amount:.2}"),
        confidence_pct: row.confidence_pct,
        first_seen_at: row.first_seen_at.to_string(),
        last_seen_at: row.last_seen_at.to_string(),
        confirmed_at: row.confirmed_at.map(|t| t.to_rfc3339()),
        rejected_at: row.rejected_at.map(|t| t.to_rfc3339()),
        transaction_count: None,
    }
}

fn map_detail(row: PatternDetailRow) -> PatternResponse {
    PatternResponse {
        id: row.id.to_string(),
        fingerprint: row.fingerprint,
        status: row.status,
        kind: row.kind,
        payee_key: row.payee_key,
        display_name: row.display_name,
        interval_days: row.interval_days,
        current_amount: format!("{:.2}", row.current_amount),
        confidence_pct: row.confidence_pct,
        first_seen_at: row.first_seen_at.to_string(),
        last_seen_at: row.last_seen_at.to_string(),
        confirmed_at: row.confirmed_at.map(|t| t.to_rfc3339()),
        rejected_at: row.rejected_at.map(|t| t.to_rfc3339()),
        transaction_count: Some(row.transaction_count),
    }
}

async fn list_patterns(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<PatternResponse>>, StatusCode> {
    let repo = state.subscriptions.repository();
    let rows = repo
        .list_patterns(q.status.as_deref(), q.kind.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(map_pattern).collect()))
}

async fn get_pattern(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PatternResponse>, StatusCode> {
    let repo = state.subscriptions.repository();
    let row = repo
        .get_pattern(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(map_detail(row)))
}

async fn confirm_pattern(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    body: Option<Json<ConfirmBody>>,
) -> Result<Json<PatternResponse>, StatusCode> {
    let kind = body.and_then(|b| b.kind.clone());
    let repo = state.subscriptions.repository();
    let row = repo
        .confirm_pattern(id, kind.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(map_pattern(row)))
}

async fn reject_pattern(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    body: Option<Json<RejectBody>>,
) -> Result<Json<PatternResponse>, StatusCode> {
    let reason = body.and_then(|b| b.reason.clone());
    let repo = state.subscriptions.repository();
    let row = repo
        .reject_pattern(id, reason.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(map_pattern(row)))
}

async fn price_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PriceHistoryResponse>, StatusCode> {
    let repo = state.subscriptions.repository();
    let events = repo
        .price_history(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(PriceHistoryResponse {
        events: events.into_iter().map(map_price_event).collect(),
    }))
}

fn map_price_event(row: PriceEventRow) -> PriceEventItem {
    PriceEventItem {
        occurred_at: row.occurred_at.to_string(),
        amount: format!("{:.2}", row.amount),
        event_type: row.event_type,
        previous_amount: row.previous_amount.map(|a| format!("{:.2}", a)),
        delta_pct: row.delta_pct.map(|d| format!("{:.2}", d)),
    }
}

async fn list_alerts(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AlertsQuery>,
) -> Result<Json<Vec<AlertRow>>, StatusCode> {
    let unread = q.unread.unwrap_or(false);
    let repo = state.subscriptions.repository();
    let rows = repo
        .list_alerts(unread)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

async fn mark_alert_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = state.subscriptions.repository();
    let updated = repo
        .mark_alert_read(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if updated {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
