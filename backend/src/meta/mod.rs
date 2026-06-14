use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct BuildInfoResponse {
    pub build_id: &'static str,
    pub release_tag: &'static str,
    pub build_timestamp: &'static str,
}

pub async fn build_info() -> Json<BuildInfoResponse> {
    Json(BuildInfoResponse {
        build_id: option_env!("BUILD_ID").unwrap_or("dev"),
        release_tag: option_env!("RELEASE_TAG").unwrap_or("dev"),
        build_timestamp: option_env!("BUILD_TIMESTAMP").unwrap_or("unknown"),
    })
}

pub fn routes() -> Router {
    Router::new().route("/api/v1/meta/build-info", get(build_info))
}
