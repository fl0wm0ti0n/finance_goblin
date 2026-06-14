use reqwest::Method;
use serde_json::json;
use sqlx::PgPool;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use flow_finance_ai::config::AppConfig;
use flow_finance_ai::firefly::{FireflyClient, FireflyError};

fn test_config(base_url: &str) -> AppConfig {
    AppConfig {
        server: flow_finance_ai::config::ServerConfig {
            host: "127.0.0.1".into(),
            port: 8080,
        },
        database: flow_finance_ai::config::DatabaseConfig {
            mode: "external".into(),
            host: "localhost".into(),
            port: 5432,
            name: "test".into(),
            user: "test".into(),
            password: "test".into(),
            max_connections: 2,
            startup_retry_initial_ms: 100,
            startup_retry_max_ms: 200,
            startup_retry_total_ms: 500,
        },
        firefly: flow_finance_ai::config::FireflyConfig {
            base_url: base_url.into(),
            personal_access_token: "test-pat".into(),
            page_limit: 500,
            audit_enabled: false,
        },
        sync: flow_finance_ai::config::SyncConfig {
            interval_seconds: 3600,
            overlap_days: 7,
        },
        oidc: flow_finance_ai::config::OidcConfig {
            issuer_url: String::new(),
            audience: String::new(),
            dev_bypass: true,
        },
        forecast: flow_finance_ai::config::ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        },
        subscriptions: flow_finance_ai::config::SubscriptionsConfig::default(),
        plans: flow_finance_ai::config::PlansConfig::default(),
        alerts: flow_finance_ai::config::AlertsConfig::default(),
        wealth: flow_finance_ai::config::WealthConfig::default(),
    }
}

fn empty_page() -> serde_json::Value {
    json!({
        "data": [],
        "meta": { "pagination": { "total_pages": 1 } },
        "links": { "next": null }
    })
}

#[tokio::test]
async fn firefly_client_rejects_non_get_methods() {
    assert!(matches!(
        FireflyClient::reject_non_get("POST"),
        Err(FireflyError::MethodNotAllowed(_))
    ));
    assert!(FireflyClient::reject_non_get("GET").is_ok());
}

#[tokio::test]
async fn sync_uses_get_only_against_mock_firefly() {
    let mock = MockServer::start().await;

    for endpoint in [
        "/api/v1/accounts",
        "/api/v1/categories",
        "/api/v1/budgets",
        "/api/v1/tags",
        "/api/v1/piggy_banks",
    ] {
        Mock::given(method("GET"))
            .and(path_regex(format!("^{endpoint}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(empty_page()))
            .mount(&mock)
            .await;
    }

    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_page()))
        .mount(&mock)
        .await;

    Mock::given(method("POST"))
        .and(path_regex(r"^/api/v1/.*"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock)
        .await;

    let pool = PgPool::connect_lazy("postgres://test:test@localhost/test_db").unwrap();
    let cfg = test_config(&mock.uri());
    let client = FireflyClient::new(&cfg.firefly, pool.clone());

    let _ = flow_finance_ai::firefly::sync::sync_reference_entities(&client, &pool).await;
    let _ =
        flow_finance_ai::firefly::sync::sync_transactions(&client, &pool, 7, "scheduled").await;

    let received = mock.received_requests().await.unwrap();
    assert!(!received.is_empty(), "expected GET requests to mock Firefly");
    for req in &received {
        assert_eq!(
            req.method,
            Method::GET,
            "non-GET request detected: {:?} {}",
            req.method,
            req.url
        );
    }
}
