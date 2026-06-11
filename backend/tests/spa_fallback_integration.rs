//! BUG-0016 / DEC-0104 (AX2): SPA fallback integration.
//! Mirrors `build_router` merge order: health → grafana proxy → API → SPA fallback.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::get;
use axum::{Json, Router};
use flow_finance_ai::analytics::grafana_routes;
use flow_finance_ai::attach_spa_fallback;
use flow_finance_ai::config::validate_grafana_upstream;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/spa");

fn fixture_static_dir() -> PathBuf {
    PathBuf::from(FIXTURES)
}

/// Test router with the same prefix ordering as `build_router` (DEC-0057 + DEC-0104).
fn test_router(static_dir: &Path, grafana_upstream: url::Url) -> Router {
    let health = Router::new().route("/health", get(|| async { Json(json!({"status": "ok"})) }));
    let analytics = grafana_routes(grafana_upstream);
    let api = Router::new().route(
        "/api/v1/nonexistent",
        get(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not found"})),
            )
        }),
    );

    let router = Router::new()
        .merge(health)
        .merge(analytics)
        .merge(api);

    attach_spa_fallback(router, static_dir)
}

async fn response_body(resp: axum::response::Response) -> String {
    String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes()
            .to_vec(),
    )
    .expect("utf8")
}

#[tokio::test]
async fn deep_links_return_200_html_shell() {
    let mock = MockServer::start().await;
    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = test_router(&fixture_static_dir(), upstream);

    for path in [
        "/forecast",
        "/subscriptions",
        "/planning",
        "/sync",
        "/analytics/cashflow",
        "/callback",
    ] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "path {path}");
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/html"),
            "path {path} expected text/html, got {ct}"
        );
        let body = response_body(resp).await;
        assert!(
            body.contains(r#"<div id="root">"#),
            "path {path} missing SPA shell marker"
        );
    }
}

#[tokio::test]
async fn api_paths_return_json_not_html() {
    let mock = MockServer::start().await;
    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = test_router(&fixture_static_dir(), upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("application/json"),
        "expected JSON content-type, got {ct}"
    );
    let body = response_body(resp).await;
    assert!(
        !body.contains(r#"<div id="root">"#),
        "API path must not return SPA index.html"
    );
}

#[tokio::test]
async fn health_returns_json_not_html() {
    let mock = MockServer::start().await;
    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = test_router(&fixture_static_dir(), upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_body(resp).await;
    assert!(body.contains("ok"));
    assert!(!body.contains(r#"<div id="root">"#));
}

#[tokio::test]
async fn grafana_proxy_not_spa_html() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("grafana-ok"))
        .mount(&mock)
        .await;

    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = test_router(&fixture_static_dir(), upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/analytics/grafana/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_body(resp).await;
    assert_eq!(body, "grafana-ok");
    assert!(!body.contains(r#"<div id="root">"#));
}

#[tokio::test]
async fn static_assets_served_with_js_content_type() {
    let mock = MockServer::start().await;
    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = test_router(&fixture_static_dir(), upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/assets/fixture.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("javascript") || ct.contains("text/javascript"),
        "expected JS content-type, got {ct}"
    );
    let body = response_body(resp).await;
    assert!(body.contains("fixture"));
    assert!(!body.contains(r#"<div id="root">"#));
}
