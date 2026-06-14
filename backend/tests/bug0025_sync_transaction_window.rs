//! BUG-0025 / DEC-0002 — manual 365d lookback vs scheduled overlap window.

use chrono::{Duration, Utc};
use flow_finance_ai::config::{
    AppConfig, DatabaseConfig, ExchangesConfig, FireflyConfig, ForecastConfig, OidcConfig,
    ServerConfig, SyncConfig,
};
use flow_finance_ai::firefly::{sync as firefly_sync, FireflyClient};
use serde_json::json;
use sqlx::PgPool;
use wiremock::matchers::{method, path_regex};
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
            audit_enabled: false,
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
            ai_bucket_min_confidence: 0.75,
        },
        subscriptions: flow_finance_ai::config::SubscriptionsConfig::default(),
        plans: flow_finance_ai::config::PlansConfig::default(),
        alerts: flow_finance_ai::config::AlertsConfig::default(),
        wealth: flow_finance_ai::config::WealthConfig::default(),
        ai: flow_finance_ai::config::AiConfig::default(),
        privacy: flow_finance_ai::config::PrivacyConfig::default(),
        exchanges: ExchangesConfig::default(),
        portfolio: flow_finance_ai::config::PortfolioConfig::default(),
        forecast_ml: flow_finance_ai::config::ForecastMlConfig::default(),
        analytics: flow_finance_ai::config::AnalyticsConfig {
            grafana_upstream: "http://grafana:3000".into(),
        },
        database_bootstrap_url: None,
    }
}

fn empty_page() -> serde_json::Value {
    json!({
        "data": [],
        "meta": { "pagination": { "total_pages": 1 } },
        "links": { "next": null }
    })
}

fn backdated_tx_page(firefly_id: &str, date: &str) -> serde_json::Value {
    json!({
        "data": [{
            "type": "transactions",
            "id": firefly_id,
            "attributes": {
                "transactions": [{
                    "date": date,
                    "amount": "-50.00",
                    "type": "withdrawal",
                    "description": "BUG0025 Stromkosten",
                    "category_id": "146",
                    "source_id": "bug0025-acct"
                }]
            }
        }],
        "meta": { "pagination": { "total_pages": 1 } },
        "links": { "next": null }
    })
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_watermark(pool: &PgPool, watermark: chrono::DateTime<Utc>) {
    sqlx::query(
        r#"
        INSERT INTO sync_cursors (entity_type, last_successful_sync_at, records_synced)
        VALUES ('transactions', $1, 100)
        ON CONFLICT (entity_type) DO UPDATE SET
            last_successful_sync_at = EXCLUDED.last_successful_sync_at,
            records_synced = EXCLUDED.records_synced
        "#,
    )
    .bind(watermark)
    .execute(pool)
    .await
    .expect("seed watermark");
}

async fn mount_reference_mocks(mock: &MockServer) {
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
            .mount(mock)
            .await;
    }
}

fn transactions_start_param(req: &wiremock::Request) -> Option<String> {
    req.url.query_pairs().find(|(k, _)| k == "start").map(|(_, v)| v.into_owned())
}

#[tokio::test]
async fn scheduled_sync_uses_watermark_minus_overlap_start() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for integration test");
            return;
        }
    };

    let watermark = Utc::now() - Duration::days(1);
    seed_watermark(&pool, watermark).await;

    let expected_start = (watermark - Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    let mock = MockServer::start().await;
    mount_reference_mocks(&mock).await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_page()))
        .mount(&mock)
        .await;

    let cfg = test_config(&mock.uri());
    let client = FireflyClient::new(&cfg.firefly, pool.clone());

    firefly_sync::sync_transactions(&client, &pool, 7, "scheduled")
        .await
        .expect("scheduled transaction sync");

    let received = mock.received_requests().await.expect("requests");
    let tx_req = received
        .iter()
        .find(|r| r.url.path().starts_with("/api/v1/transactions"))
        .expect("transactions GET");
    assert_eq!(
        transactions_start_param(tx_req).as_deref(),
        Some(expected_start.as_str()),
        "scheduled sync must use watermark − overlap_days start"
    );
}

#[tokio::test]
async fn manual_sync_uses_365_day_lookback_and_ingests_backdated_tx() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for integration test");
            return;
        }
    };

    let watermark = Utc::now() - Duration::days(1);
    seed_watermark(&pool, watermark).await;

    let backdated_date = (Utc::now() - Duration::days(30)).format("%Y-%m-%d").to_string();
    let firefly_id = format!("bug0025-manual-{}", Utc::now().timestamp_millis());

    let expected_start = (Utc::now() - Duration::days(365))
        .format("%Y-%m-%d")
        .to_string();

    let mock = MockServer::start().await;
    mount_reference_mocks(&mock).await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(backdated_tx_page(
            &firefly_id,
            &backdated_date,
        )))
        .mount(&mock)
        .await;

    let cfg = test_config(&mock.uri());
    let client = FireflyClient::new(&cfg.firefly, pool.clone());

    firefly_sync::sync_transactions(&client, &pool, 7, "manual")
        .await
        .expect("manual transaction sync");

    let received = mock.received_requests().await.expect("requests");
    let tx_req = received
        .iter()
        .find(|r| r.url.path().starts_with("/api/v1/transactions"))
        .expect("transactions GET");
    assert_eq!(
        transactions_start_param(tx_req).as_deref(),
        Some(expected_start.as_str()),
        "manual sync must use 365-day lookback start"
    );

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE firefly_id = $1")
        .bind(&firefly_id)
        .fetch_one(&pool)
        .await
        .expect("count backdated tx");
    assert_eq!(count, 1, "manual sync must ingest backdated transaction");
}

#[tokio::test]
async fn scheduled_sync_does_not_ingest_when_firefly_returns_empty_for_narrow_window() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for integration test");
            return;
        }
    };

    let watermark = Utc::now() - Duration::days(1);
    seed_watermark(&pool, watermark).await;

    let firefly_id = format!("bug0025-sched-empty-{}", Utc::now().timestamp_millis());

    let mock = MockServer::start().await;
    mount_reference_mocks(&mock).await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v1/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_page()))
        .mount(&mock)
        .await;

    let cfg = test_config(&mock.uri());
    let client = FireflyClient::new(&cfg.firefly, pool.clone());

    firefly_sync::sync_transactions(&client, &pool, 7, "scheduled")
        .await
        .expect("scheduled transaction sync");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE firefly_id = $1")
        .bind(&firefly_id)
        .fetch_one(&pool)
        .await
        .expect("count missing tx");
    assert_eq!(
        count, 0,
        "scheduled sync with empty Firefly response must not create mirror rows"
    );
}
