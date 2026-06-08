use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::subscriptions::types::OperatorTagRow;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateTagBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameTagBody {
    pub name: String,
}

#[derive(Serialize)]
struct TagResponse {
    id: String,
    name: String,
    slug: String,
    created_at: String,
    updated_at: String,
}

fn map_tag(row: OperatorTagRow) -> TagResponse {
    TagResponse {
        id: row.id.to_string(),
        name: row.name,
        slug: row.slug,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/subscription-tags", get(list_tags).post(create_tag))
        .route(
            "/api/v1/subscription-tags/:id",
            patch(rename_tag).delete(delete_tag),
        )
}

async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagResponse>>, StatusCode> {
    let repo = state.subscriptions.repository();
    let rows = repo
        .list_operator_tags()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(map_tag).collect()))
}

async fn create_tag(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTagBody>,
) -> Result<(StatusCode, Json<TagResponse>), StatusCode> {
    let repo = state.subscriptions.repository();
    let row = repo
        .create_operator_tag(&body.name)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            }
        })?;
    Ok((StatusCode::CREATED, Json(map_tag(row))))
}

async fn rename_tag(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<RenameTagBody>,
) -> Result<Json<TagResponse>, StatusCode> {
    let repo = state.subscriptions.repository();
    let row = repo
        .rename_operator_tag(id, &body.name)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            }
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(map_tag(row)))
}

async fn delete_tag(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = state.subscriptions.repository();
    let deleted = repo
        .delete_operator_tag(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db| db.code())
        .map(|code| code == "23505")
        .unwrap_or(false)
}
