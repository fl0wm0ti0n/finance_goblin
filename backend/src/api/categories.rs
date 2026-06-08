use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::transactions::TransactionsError;
use crate::transactions::types::{
    CategoryCatalogResponse, ExpenseSeriesResponse, EXPENSE_SERIES_DEFAULT_MONTHS,
};
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/categories", get(list_categories))
        .route("/api/v1/categories/expense-series", get(expense_series))
}

#[derive(Deserialize)]
struct CatalogQuery {
    q: Option<String>,
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<CategoryCatalogResponse>, (StatusCode, Json<serde_json::Value>)> {
    state
        .transactions
        .list_categories_catalog(query.q.as_deref())
        .await
        .map(Json)
        .map_err(map_transactions_error)
}

#[derive(Deserialize)]
struct ExpenseSeriesQuery {
    category_id: String,
    #[serde(default = "default_months")]
    months: u32,
    end: Option<NaiveDate>,
}

fn default_months() -> u32 {
    EXPENSE_SERIES_DEFAULT_MONTHS
}

async fn expense_series(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExpenseSeriesQuery>,
) -> Result<Json<ExpenseSeriesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if query.category_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "category_id is required"
            })),
        ));
    }

    let end = query.end.unwrap_or_else(|| chrono::Local::now().date_naive());

    state
        .transactions
        .expense_series(&query.category_id, query.months, end)
        .await
        .map(Json)
        .map_err(map_transactions_error)
}

fn map_transactions_error(err: TransactionsError) -> (StatusCode, Json<serde_json::Value>) {
    match err {
        TransactionsError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "category not found",
                "category_id": id,
                "hint": "Category may have been deleted in Firefly — run a Full sync or pick another category"
            })),
        ),
        TransactionsError::InvalidArgs(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        ),
        TransactionsError::Db(e) => {
            tracing::error!("categories api db error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal server error" })),
            )
        }
    }
}
