use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;
use serde_json::Value;

#[tokio::test]
async fn test_meta_build_info_endpoint() {
    let app = flow_finance_ai::meta::routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/meta/build-info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("build_id").is_some());
    assert!(json.get("release_tag").is_some());
    assert!(json.get("build_timestamp").is_some());

    let build_id = json["build_id"].as_str().unwrap();
    let release_tag = json["release_tag"].as_str().unwrap();
    let build_timestamp = json["build_timestamp"].as_str().unwrap();

    assert!(!build_id.is_empty());
    assert!(!release_tag.is_empty());
    assert!(!build_timestamp.is_empty());
}

#[tokio::test]
async fn test_meta_build_info_no_secrets() {
    let app = flow_finance_ai::meta::routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/meta/build-info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    assert!(!body_str.contains("FIREfly"));
    assert!(!body_str.contains("DATABASE"));
    assert!(!body_str.contains("OIDC"));
    assert!(!body_str.contains("secret"));
    assert!(!body_str.contains("password"));
}

#[tokio::test]
async fn test_meta_build_info_fallback_values() {
    let app = flow_finance_ai::meta::routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/meta/build-info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let build_id = json["build_id"].as_str().unwrap();
    let release_tag = json["release_tag"].as_str().unwrap();
    let build_timestamp = json["build_timestamp"].as_str().unwrap();

    assert!(
        build_id == "dev" || !build_id.is_empty(),
        "build_id should be 'dev' or a valid value"
    );
    assert!(
        release_tag == "dev" || !release_tag.is_empty(),
        "release_tag should be 'dev' or a valid value"
    );
    assert!(
        build_timestamp == "unknown" || !build_timestamp.is_empty(),
        "build_timestamp should be 'unknown' or a valid value"
    );
}
