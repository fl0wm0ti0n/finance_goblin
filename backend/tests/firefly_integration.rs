use std::sync::Arc;

use flow_finance_ai::config::{AppConfig, DatabaseConfig, FireflyConfig, ForecastConfig, OidcConfig, ServerConfig, SyncConfig};
use flow_finance_ai::firefly::{sync as firefly_sync, FireflyClient};
use serde_json::json;
use sqlx::PgPool;
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config(base_url: &str) -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 8080,
        },
        database: DatabaseConfig {
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
        firefly: FireflyConfig {
            base_url: base_url.into(),
            personal_access_token: "test-pat".into(),
            page_limit: 500,
            audit_enabled: true,
        },
        sync: SyncConfig {
            interval_seconds: 3600,
            overlap_days: 7,
        },
        oidc: OidcConfig {
            issuer_url: String::new(),
            audience: String::new(),
            dev_bypass: true,
        },
        forecast: ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
        },
        subscriptions: flow_finance_ai::config::SubscriptionsConfig::default(),
        plans: flow_finance_ai::config::PlansConfig::default(),
    }
}

fn empty_list_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "data": [],
        "meta": { "pagination": { "total_pages": 1 } }
    }))
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    sqlx::query("TRUNCATE firefly_request_audit RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .ok()?;
    Some(pool)
}

#[tokio::test]
async fn sync_issues_only_get_requests_to_firefly() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for integration test");
            return;
        }
    };

    let mock = MockServer::start().await;
    let endpoints = [
        "/api/v1/accounts",
        "/api/v1/categories",
        "/api/v1/budgets",
        "/api/v1/tags",
        "/api/v1/piggy_banks",
        "/api/v1/transactions",
    ];

    for ep in endpoints {
        Mock::given(method("GET"))
            .and(path(ep))
            .and(header("Authorization", "Bearer test-pat"))
            .respond_with(empty_list_response())
            .mount(&mock)
            .await;

        for bad_method in ["POST", "PUT", "PATCH", "DELETE"] {
            Mock::given(method(bad_method))
                .and(path(ep))
                .respond_with(ResponseTemplate::new(500).set_body_string("WRITE FORBIDDEN"))
                .mount(&mock)
                .await;
        }
    }

    // Transactions may include query string
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v1/transactions(\?.*)?$"))
        .and(header("Authorization", "Bearer test-pat"))
        .respond_with(empty_list_response())
        .mount(&mock)
        .await;

    let cfg = test_config(&mock.uri());
    let client = FireflyClient::new(&cfg.firefly, pool.clone());

    firefly_sync::sync_reference_entities(&client, &pool)
        .await
        .expect("reference sync");
    firefly_sync::sync_transactions(&client, &pool, 7)
        .await
        .expect("transaction sync");

    let non_get: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM firefly_request_audit WHERE method <> 'GET'",
    )
    .fetch_one(&pool)
    .await
    .expect("audit query");

    assert_eq!(
        non_get, 0,
        "integration test failed: non-GET Firefly requests recorded in audit log"
    );

    let _ = Arc::new(cfg);
}
