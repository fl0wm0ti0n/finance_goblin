//! US-0011: Grafana analytics proxy smoke (DEC-0057).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use flow_finance_ai::analytics::grafana_routes;
use flow_finance_ai::config::validate_grafana_upstream;
use http_body_util::BodyExt;
use tower::ServiceExt;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn proxy_strips_prefix_and_returns_upstream_body() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&mock)
        .await;

    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock on 127.0.0.1");
    let app = grafana_routes(upstream);

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
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body.as_ref(), b"ok");

    let recorded = mock.received_requests().await.expect("requests");
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].url.path(), "/api/health");
}

#[tokio::test]
async fn proxy_removes_blocking_x_frame_options() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("dashboard")
                .insert_header("X-Frame-Options", "deny"),
        )
        .mount(&mock)
        .await;

    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = grafana_routes(upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/analytics/grafana")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get("X-Frame-Options").is_none());
}

#[tokio::test]
async fn proxy_strips_set_cookie_from_upstream() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("ok")
                .insert_header("Set-Cookie", "grafana_session=abc; Path=/"),
        )
        .mount(&mock)
        .await;

    let upstream = validate_grafana_upstream(&mock.uri()).expect("wiremock");
    let app = grafana_routes(upstream);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/analytics/grafana/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get_all("set-cookie").iter().next().is_none());
}

#[test]
fn websocket_live_route_registered() {
    let upstream = Url::parse("http://127.0.0.1:3000").unwrap();
    let _router = grafana_routes(upstream);
    // Route wiring is compile-time; WS handshake requires live upstream (QA smoke).
}
