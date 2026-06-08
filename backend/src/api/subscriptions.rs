use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::subscriptions::discovery::{run_discover, DiscoverQuery};
use crate::subscriptions::types::{
    AlertRow, ConfirmFromDiscoverError, OperatorTagSummary, PatternDetailRow, PatternRow,
    PriceEventRow,
};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub kind: Option<String>,
    pub tag: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscoverQueryParams {
    pub account_id: Option<String>,
    pub payee: Option<String>,
    pub interval_days: Option<i32>,
    pub limit: Option<usize>,
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

#[derive(Deserialize)]
pub struct DiscoverConfirmBody {
    pub payee_key: String,
    pub interval_days: i32,
    pub median_amount: f64,
    pub transaction_ids: Vec<String>,
    pub kind: Option<String>,
}

#[derive(Deserialize)]
pub struct AssignTagsBody {
    pub tag_ids: Vec<String>,
}

#[derive(Serialize)]
struct TagItem {
    id: String,
    name: String,
    slug: String,
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
    display_category_id: Option<String>,
    transaction_count: Option<i64>,
    tags: Vec<TagItem>,
}

#[derive(Serialize)]
struct DiscoverConfirmResponse {
    pattern: PatternResponse,
    merged: bool,
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
        .route("/api/v1/subscriptions/discover", get(discover_patterns))
        .route(
            "/api/v1/subscriptions/discover/confirm",
            post(confirm_discover),
        )
        .route("/api/v1/subscriptions/alerts", get(list_alerts))
        .route("/api/v1/subscriptions/alerts/unread-count", get(unread_alert_count))
        .route("/api/v1/subscriptions/alerts/:id/read", axum::routing::patch(mark_alert_read))
        .route("/api/v1/subscriptions/:id", get(get_pattern))
        .route("/api/v1/subscriptions/:id/confirm", post(confirm_pattern))
        .route("/api/v1/subscriptions/:id/reject", post(reject_pattern))
        .route("/api/v1/subscriptions/:id/tags", put(assign_tags))
        .route("/api/v1/subscriptions/:id/price-history", get(price_history))
}

fn map_tags(tags: Vec<OperatorTagSummary>) -> Vec<TagItem> {
    tags.into_iter()
        .map(|t| TagItem {
            id: t.id.to_string(),
            name: t.name,
            slug: t.slug,
        })
        .collect()
}

fn map_pattern(row: PatternRow, tags: Vec<OperatorTagSummary>) -> PatternResponse {
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
        display_category_id: row.display_category_id,
        transaction_count: None,
        tags: map_tags(tags),
    }
}

fn map_detail(row: PatternDetailRow, tags: Vec<OperatorTagSummary>) -> PatternResponse {
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
        display_category_id: row.display_category_id,
        transaction_count: Some(row.transaction_count),
        tags: map_tags(tags),
    }
}

async fn list_patterns(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<PatternResponse>>, StatusCode> {
    let repo = state.subscriptions.repository();
    let rows = repo
        .list_patterns(q.status.as_deref(), q.kind.as_deref(), q.tag.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let tag_map = repo
        .list_tags_for_patterns(&ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| {
                let tags = tag_map.get(&r.id).cloned().unwrap_or_default();
                map_pattern(r, tags)
            })
            .collect(),
    ))
}

async fn discover_patterns(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DiscoverQueryParams>,
) -> Result<Json<crate::subscriptions::types::DiscoverResponse>, StatusCode> {
    let limit = q.limit.unwrap_or(50).min(50).max(1);
    let response = run_discover(
        state.subscriptions.repository(),
        DiscoverQuery {
            account_id: q.account_id.as_deref(),
            payee: q.payee.as_deref(),
            interval_days: q.interval_days,
            limit,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(response))
}

async fn confirm_discover(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DiscoverConfirmBody>,
) -> Result<(StatusCode, Json<DiscoverConfirmResponse>), StatusCode> {
    if body.transaction_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let kind = body.kind.as_deref().unwrap_or("subscription");
    let sync_run_id = Uuid::new_v4();
    let repo = state.subscriptions.repository();
    let result = repo
        .confirm_from_discover(
            &body.payee_key,
            body.interval_days,
            body.median_amount,
            &body.transaction_ids,
            kind,
            sync_run_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Ok(ok) => {
            let tags = repo
                .list_tags_for_patterns(&[ok.pattern.id])
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .remove(&ok.pattern.id)
                .unwrap_or_default();
            let status = if ok.merged {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            Ok((
                status,
                Json(DiscoverConfirmResponse {
                    pattern: map_pattern(ok.pattern, tags),
                    merged: ok.merged,
                }),
            ))
        }
        Err(ConfirmFromDiscoverError::RejectedPayeeInterval) => Err(StatusCode::CONFLICT),
        Err(ConfirmFromDiscoverError::FingerprintConflict) => Err(StatusCode::CONFLICT),
        Err(ConfirmFromDiscoverError::InvalidTransactions(_)) => Err(StatusCode::BAD_REQUEST),
    }
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
    let tags = repo
        .list_tags_for_patterns(&[id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&id)
        .unwrap_or_default();
    Ok(Json(map_detail(row, tags)))
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
    let tags = repo
        .list_tags_for_patterns(&[id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&id)
        .unwrap_or_default();
    Ok(Json(map_pattern(row, tags)))
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
    let tags = repo
        .list_tags_for_patterns(&[id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&id)
        .unwrap_or_default();
    Ok(Json(map_pattern(row, tags)))
}

async fn assign_tags(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignTagsBody>,
) -> Result<Json<Vec<TagItem>>, StatusCode> {
    let tag_ids: Vec<Uuid> = body
        .tag_ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let repo = state.subscriptions.repository();
    let pattern = repo
        .get_pattern(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if pattern.status != "confirmed" && pattern.status != "pending" {
        return Err(StatusCode::BAD_REQUEST);
    }

    repo.replace_pattern_tags(id, &tag_ids)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let tags = repo
        .list_tags_for_patterns(&[id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&id)
        .unwrap_or_default();
    Ok(Json(map_tags(tags)))
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

async fn unread_alert_count(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::subscriptions::types::UnreadAlertCountResponse>, StatusCode> {
    let counts = state
        .subscriptions
        .unread_alert_counts()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(counts))
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
