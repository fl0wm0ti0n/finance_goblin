use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::plan::service::PlanError;
use crate::plan::templates::TemplateOverrides;
use crate::plan::types::{
    AdjustmentDirection, AdjustmentFrequency, AdjustmentTarget, PlanAdjustment, SavingsSuggestion,
};
use crate::AppState;

#[derive(Deserialize)]
pub struct CreatePlanBody {
    pub name: String,
    pub template: Option<String>,
}

#[derive(Deserialize)]
pub struct RenamePlanBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ApplyTemplateBody {
    pub template: String,
    pub subscription_payee_keys: Option<Vec<String>>,
    pub discretionary_cut: Option<bool>,
}

#[derive(Deserialize)]
pub struct AdjustmentBody {
    pub direction: String,
    pub amount: f64,
    pub frequency: String,
    pub target_type: String,
    pub target_key: Option<String>,
    pub label: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct PlanVsActualQuery {
    pub month: Option<String>,
}

#[derive(Serialize)]
struct PlanResponse {
    id: String,
    name: String,
    template: String,
    is_active: bool,
}

#[derive(Serialize)]
struct VersionResponse {
    id: String,
    plan_id: String,
    version_number: i32,
    is_latest: bool,
    frozen: bool,
    adjustments: Vec<AdjustmentResponse>,
}

#[derive(Serialize)]
struct AdjustmentResponse {
    id: String,
    direction: String,
    amount: String,
    frequency: String,
    target_type: String,
    target_key: Option<String>,
    label: Option<String>,
    effective_from: String,
    effective_to: Option<String>,
    sort_order: i32,
}

#[derive(Serialize)]
struct PlanDetailResponse {
    plan: PlanResponse,
    versions: Vec<VersionSummary>,
}

#[derive(Serialize)]
struct VersionSummary {
    id: String,
    version_number: i32,
    is_latest: bool,
    frozen: bool,
}

#[derive(Serialize)]
struct RecomputeResponse {
    status: &'static str,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/plans", get(list_plans).post(create_plan))
        .route("/api/v1/plans/active/plan-vs-actual", get(plan_vs_actual))
        .route("/api/v1/plans/risk-score", get(risk_score))
        .route("/api/v1/plans/templates/savings-mode/suggestions", get(savings_suggestions))
        .route("/api/v1/plans/:id", get(get_plan).patch(rename_plan).delete(delete_plan))
        .route("/api/v1/plans/:id/activate", post(activate_plan))
        .route("/api/v1/plans/:id/compare", get(compare_plan))
        .route("/api/v1/plans/:id/versions", get(list_versions).post(create_version))
        .route(
            "/api/v1/plans/:id/versions/:vid",
            get(get_version).patch(update_version_adjustments),
        )
        .route(
            "/api/v1/plans/:id/versions/:vid/adjustments",
            post(add_adjustment),
        )
        .route(
            "/api/v1/plans/:id/versions/:vid/adjustments/:aid",
            patch(update_adjustment).delete(remove_adjustment),
        )
        .route(
            "/api/v1/plans/:id/versions/:vid/apply-template",
            post(apply_template),
        )
        .route(
            "/api/v1/plans/:id/versions/:vid/recompute",
            post(manual_recompute),
        )
}

fn map_plan(row: crate::plan::types::PlanRow) -> PlanResponse {
    PlanResponse {
        id: row.id.to_string(),
        name: row.name,
        template: row.template,
        is_active: row.is_active,
    }
}

fn map_adjustment(adj: &PlanAdjustment) -> AdjustmentResponse {
    AdjustmentResponse {
        id: adj.id.to_string(),
        direction: adj.direction.as_str().into(),
        amount: format!("{:.2}", adj.amount),
        frequency: adj.frequency.as_str().into(),
        target_type: adj.target_type.as_str().into(),
        target_key: adj.target_key.clone(),
        label: adj.label.clone(),
        effective_from: adj.effective_from.to_string(),
        effective_to: adj.effective_to.map(|d| d.to_string()),
        sort_order: adj.sort_order,
    }
}

fn parse_adjustment_body(body: &AdjustmentBody, version_id: Uuid, id: Uuid) -> Result<PlanAdjustment, StatusCode> {
    let direction = AdjustmentDirection::from_str(&body.direction)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let frequency = AdjustmentFrequency::from_str(&body.frequency)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let target_type = AdjustmentTarget::from_str(&body.target_type)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let effective_from = body
        .effective_from
        .as_deref()
        .map(parse_date)
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let effective_to = body
        .effective_to
        .as_deref()
        .map(parse_date)
        .transpose()?;

    Ok(PlanAdjustment {
        id,
        version_id,
        direction,
        amount: body.amount,
        frequency,
        target_type,
        target_key: body.target_key.clone(),
        label: body.label.clone(),
        effective_from,
        effective_to,
        sort_order: body.sort_order.unwrap_or(0),
    })
}

fn parse_date(s: &str) -> Result<NaiveDate, StatusCode> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_month(s: &str) -> Result<NaiveDate, StatusCode> {
    NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d").map_err(|_| StatusCode::BAD_REQUEST)
}

fn plan_error_status(err: PlanError) -> (StatusCode, Json<serde_json::Value>) {
    match err {
        PlanError::NotFound | PlanError::VersionNotFound | PlanError::AdjustmentNotFound => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": err.to_string() })),
        ),
        PlanError::VersionCapReached | PlanError::VersionFrozen => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": err.to_string() })),
        ),
        PlanError::ActivePlanDeleteForbidden => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "active_plan_delete_forbidden",
                "message": "Cannot delete the active plan. Set another plan active first, then delete."
            })),
        ),
        PlanError::NoActivePlan => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "no_active_plan" })),
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": other.to_string() })),
        ),
    }
}

async fn list_plans(State(state): State<Arc<AppState>>) -> Result<Json<Vec<crate::plan::types::PlanListItem>>, StatusCode> {
    state
        .plans
        .list_plans()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_plan(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePlanBody>,
) -> Result<(StatusCode, Json<PlanDetailResponse>), (StatusCode, Json<serde_json::Value>)> {
    let (plan, version) = state
        .plans
        .create_plan(&body.name, body.template.as_deref())
        .await
        .map_err(plan_error_status)?;

    Ok((
        StatusCode::CREATED,
        Json(PlanDetailResponse {
            plan: map_plan(plan),
            versions: vec![VersionSummary {
                id: version.id.to_string(),
                version_number: version.version_number,
                is_latest: version.is_latest,
                frozen: version.frozen_at.is_some(),
            }],
        }),
    ))
}

async fn get_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlanDetailResponse>, StatusCode> {
    let plan = state
        .plans
        .repository()
        .get_plan(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let versions = state
        .plans
        .list_versions(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PlanDetailResponse {
        plan: map_plan(plan),
        versions: versions
            .into_iter()
            .map(|v| VersionSummary {
                id: v.id.to_string(),
                version_number: v.version_number,
                is_latest: v.is_latest,
                frozen: v.frozen_at.is_some(),
            })
            .collect(),
    }))
}

async fn rename_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<RenamePlanBody>,
) -> Result<StatusCode, StatusCode> {
    state
        .plans
        .rename_plan(id, &body.name)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state.plans.delete_plan(id).await.map_err(plan_error_status)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn activate_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .plans
        .activate_plan(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_versions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VersionSummary>>, StatusCode> {
    let versions = state
        .plans
        .list_versions(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        versions
            .into_iter()
            .map(|v| VersionSummary {
                id: v.id.to_string(),
                version_number: v.version_number,
                is_latest: v.is_latest,
                frozen: v.frozen_at.is_some(),
            })
            .collect(),
    ))
}

async fn create_version(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<VersionSummary>), (StatusCode, Json<serde_json::Value>)> {
    let version = state
        .plans
        .create_version(id)
        .await
        .map_err(plan_error_status)?;
    Ok((
        StatusCode::CREATED,
        Json(VersionSummary {
            id: version.id.to_string(),
            version_number: version.version_number,
            is_latest: version.is_latest,
            frozen: version.frozen_at.is_some(),
        }),
    ))
}

async fn get_version(
    State(state): State<Arc<AppState>>,
    Path((_id, vid)): Path<(Uuid, Uuid)>,
) -> Result<Json<VersionResponse>, StatusCode> {
    let version = state
        .plans
        .get_version(vid)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let adjustments = state
        .plans
        .load_adjustments(vid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(VersionResponse {
        id: version.id.to_string(),
        plan_id: version.plan_id.to_string(),
        version_number: version.version_number,
        is_latest: version.is_latest,
        frozen: version.frozen_at.is_some(),
        adjustments: adjustments.iter().map(map_adjustment).collect(),
    }))
}

async fn update_version_adjustments(
    State(state): State<Arc<AppState>>,
    Path((_id, vid)): Path<(Uuid, Uuid)>,
    Json(body): Json<Vec<AdjustmentBody>>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut adjustments = Vec::new();
    for (idx, item) in body.iter().enumerate() {
        let mut adj = parse_adjustment_body(item, vid, Uuid::new_v4()).map_err(|s| {
            (s, Json(serde_json::json!({ "error": "invalid adjustment" })))
        })?;
        adj.sort_order = item.sort_order.unwrap_or(idx as i32);
        adjustments.push(adj);
    }
    state
        .plans
        .repository()
        .replace_adjustments(vid, &adjustments)
        .await
        .map_err(|e| plan_error_status(e.into()))?;
    state.plans.spawn_recompute(vid);
    Ok(StatusCode::NO_CONTENT)
}

async fn add_adjustment(
    State(state): State<Arc<AppState>>,
    Path((_id, vid)): Path<(Uuid, Uuid)>,
    Json(body): Json<AdjustmentBody>,
) -> Result<(StatusCode, Json<AdjustmentResponse>), (StatusCode, Json<serde_json::Value>)> {
    let adj = parse_adjustment_body(&body, vid, Uuid::new_v4()).map_err(|s| {
        (s, Json(serde_json::json!({ "error": "invalid adjustment" })))
    })?;
    let id = state.plans.add_adjustment(vid, &adj).await.map_err(plan_error_status)?;
    let mut saved = adj;
    saved.id = id;
    Ok((StatusCode::CREATED, Json(map_adjustment(&saved))))
}

async fn update_adjustment(
    State(state): State<Arc<AppState>>,
    Path((_id, vid, aid)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<AdjustmentBody>,
) -> Result<Json<AdjustmentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let adj = parse_adjustment_body(&body, vid, aid).map_err(|s| {
        (s, Json(serde_json::json!({ "error": "invalid adjustment" })))
    })?;
    state
        .plans
        .update_adjustment(vid, aid, &adj)
        .await
        .map_err(plan_error_status)?;
    Ok(Json(map_adjustment(&adj)))
}

async fn remove_adjustment(
    State(state): State<Arc<AppState>>,
    Path((_id, vid, aid)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .plans
        .delete_adjustment(vid, aid)
        .await
        .map_err(plan_error_status)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn apply_template(
    State(state): State<Arc<AppState>>,
    Path((_id, vid)): Path<(Uuid, Uuid)>,
    Json(body): Json<ApplyTemplateBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let overrides = TemplateOverrides {
        subscription_payee_keys: body.subscription_payee_keys.unwrap_or_default(),
        discretionary_cut: body.discretionary_cut.unwrap_or(false),
    };
    state
        .plans
        .apply_template(vid, &body.template, overrides)
        .await
        .map_err(plan_error_status)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn savings_suggestions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SavingsSuggestion>>, StatusCode> {
    state
        .plans
        .savings_suggestions()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn compare_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::plan::types::CompareResponse>, StatusCode> {
    state
        .plans
        .compare_versions(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn plan_vs_actual(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PlanVsActualQuery>,
) -> Result<Json<crate::plan::types::PlanVsActualApiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let month = query
        .month
        .as_deref()
        .map(parse_month)
        .transpose()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid month format, use YYYY-MM" })),
            )
        })?;

    let active = state
        .plans
        .active_plan()
        .await
        .map_err(|e| plan_error_status(e))?;
    let Some(_active) = active else {
        return Ok(Json(crate::plan::types::PlanVsActualApiResponse::NoActivePlan {
            reason: "no_active_plan",
        }));
    };

    state
        .plans
        .plan_vs_actual(month)
        .await
        .map(crate::plan::types::PlanVsActualApiResponse::from)
        .map(Json)
        .map_err(plan_error_status)
}

async fn manual_recompute(
    State(state): State<Arc<AppState>>,
    Path((_id, vid)): Path<(Uuid, Uuid)>,
) -> Result<Json<RecomputeResponse>, (StatusCode, Json<serde_json::Value>)> {
    state.plans.spawn_recompute(vid);
    Ok(Json(RecomputeResponse { status: "accepted" }))
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum RiskScoreApiResponse {
    #[serde(rename = "ok")]
    Ok {
        score: i16,
        band: String,
        components: crate::plan::risk::RiskComponents,
        plan_computation_id: String,
    },
    #[serde(rename = "no_score")]
    NoScore { reason: &'static str },
}

async fn risk_score(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RiskScoreApiResponse>, StatusCode> {
    let active = state
        .plans
        .active_plan()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(_active) = active else {
        return Ok(Json(RiskScoreApiResponse::NoScore {
            reason: "no_active_plan",
        }));
    };

    let score = state
        .plan_risk
        .latest_for_active_plan()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match score {
        Some(score) => Ok(Json(RiskScoreApiResponse::Ok {
            score: score.score,
            band: score.band,
            components: score.components,
            plan_computation_id: score.plan_computation_id.to_string(),
        })),
        None => Ok(Json(RiskScoreApiResponse::NoScore {
            reason: "not_computed",
        })),
    }
}

#[cfg(test)]
mod plan_delete_api_tests {
    use super::plan_error_status;
    use crate::plan::service::PlanError;
    use axum::http::StatusCode;

    #[test]
    fn active_plan_delete_returns_409_with_code() {
        let (status, body) = plan_error_status(PlanError::ActivePlanDeleteForbidden);
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(
            body.0.get("error").and_then(|v| v.as_str()),
            Some("active_plan_delete_forbidden")
        );
        assert!(body.0.get("message").and_then(|v| v.as_str()).is_some());
    }
}

#[cfg(test)]
mod plan_vs_actual_api_tests {
    use crate::plan::types::PlanVsActualApiResponse;
    use serde_json::json;

    #[test]
    fn no_active_plan_serializes_with_reason() {
        let body = PlanVsActualApiResponse::NoActivePlan {
            reason: "no_active_plan",
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(
            v,
            json!({
                "status": "no_active_plan",
                "reason": "no_active_plan"
            })
        );
    }

    #[test]
    fn ok_plan_vs_actual_serializes_with_status_ok() {
        let body = PlanVsActualApiResponse::Ok {
            month: "2026-06".into(),
            reporting_currency: "EUR".into(),
            plan_stale: false,
            actuals_stale: false,
            rows: vec![],
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v.get("status").and_then(|s| s.as_str()), Some("ok"));
        assert_eq!(
            v.get("reporting_currency").and_then(|s| s.as_str()),
            Some("EUR")
        );
    }
}

#[cfg(test)]
mod risk_score_tests {
    use super::RiskScoreApiResponse;
    use serde_json::json;

    #[test]
    fn no_score_serializes_with_reason() {
        let body = RiskScoreApiResponse::NoScore {
            reason: "no_active_plan",
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(
            v,
            json!({
                "status": "no_score",
                "reason": "no_active_plan"
            })
        );
    }

    #[test]
    fn ok_score_serializes_with_status_ok() {
        let body = RiskScoreApiResponse::Ok {
            score: 42,
            band: "medium".into(),
            components: crate::plan::risk::RiskComponents {
                balance_stress: 10.0,
                plan_viability: 20.0,
                crypto_volatility: 5.0,
                ml_divergence_modifier: 0.0,
            },
            plan_computation_id: "00000000-0000-0000-0000-000000000001".into(),
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v.get("status").and_then(|s| s.as_str()), Some("ok"));
        assert_eq!(v.get("score").and_then(|s| s.as_i64()), Some(42));
        assert!(v.get("reason").is_none());
    }
}
