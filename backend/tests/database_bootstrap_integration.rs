//! US-0012: Database bootstrap integration (DEC-0058 / AC-6).
//! Gated on `DATABASE_BOOTSTRAP_TEST_URL` (superuser maintenance access).

use flow_finance_ai::config::{
    AppConfig, DatabaseConfig, ExchangesConfig, FireflyConfig, ForecastConfig, OidcConfig,
    PlansConfig, PortfolioConfig, ServerConfig, SubscriptionsConfig, SyncConfig,
};
use flow_finance_ai::db::bootstrap::ensure_database;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn bootstrap_test_url() -> Option<String> {
    std::env::var("DATABASE_BOOTSTRAP_TEST_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn test_config(db_name: &str, bootstrap_url: &str) -> AppConfig {
    let parsed = url::Url::parse(bootstrap_url).expect("DATABASE_BOOTSTRAP_TEST_URL");
    let host = parsed.host_str().unwrap_or("localhost").to_string();
    let port = parsed.port().unwrap_or(5432);

    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 8080,
        },
        database: DatabaseConfig {
            mode: "external".into(),
            host,
            port,
            name: db_name.to_string(),
            user: "finance".into(),
            password: "bootstrap-test".into(),
            max_connections: 2,
            startup_retry_initial_ms: 200,
            startup_retry_max_ms: 1000,
            startup_retry_total_ms: 15000,
        },
        firefly: FireflyConfig {
            base_url: "http://localhost".into(),
            personal_access_token: "test".into(),
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
        },
        subscriptions: SubscriptionsConfig::default(),
        plans: PlansConfig::default(),
        alerts: flow_finance_ai::config::AlertsConfig::default(),
        wealth: flow_finance_ai::config::WealthConfig::default(),
        ai: flow_finance_ai::config::AiConfig::default(),
        privacy: flow_finance_ai::config::PrivacyConfig::default(),
        exchanges: ExchangesConfig::default(),
        portfolio: PortfolioConfig::default(),
        forecast_ml: flow_finance_ai::config::ForecastMlConfig::default(),
        analytics: flow_finance_ai::config::AnalyticsConfig {
            grafana_upstream: "http://grafana:3000".into(),
        },
        database_bootstrap_url: Some(bootstrap_url.to_string()),
    }
}

async fn drop_database_if_exists(maintenance_url: &str, db_name: &str) {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(maintenance_url)
        .await
        .expect("maintenance connect for teardown");
    let sql = format!(r#"DROP DATABASE IF EXISTS "{db_name}""#);
    let _ = sqlx::query(&sql).execute(&pool).await;
    pool.close().await;
}

#[tokio::test]
async fn bootstrap_creates_missing_database_idempotently() {
    let Some(maintenance_url) = bootstrap_test_url() else {
        eprintln!("SKIP database_bootstrap_integration: set DATABASE_BOOTSTRAP_TEST_URL");
        return;
    };

    let db_name = format!("bootstrap_test_{}", Uuid::new_v4().simple());
    drop_database_if_exists(&maintenance_url, &db_name).await;

    let config = test_config(&db_name, &maintenance_url);

    ensure_database(&config)
        .await
        .expect("first bootstrap should create database and extension");

    let maint = PgPoolOptions::new()
        .max_connections(1)
        .connect(&maintenance_url)
        .await
        .expect("maintenance pool");
    let exists: (i32,) = sqlx::query_as("SELECT 1 FROM pg_database WHERE datname = $1")
        .bind(&db_name)
        .fetch_one(&maint)
        .await
        .expect("database row");
    assert_eq!(exists.0, 1);
    maint.close().await;

    ensure_database(&config)
        .await
        .expect("second bootstrap should skip create (idempotent)");

    drop_database_if_exists(&maintenance_url, &db_name).await;
}

#[tokio::test]
async fn bootstrap_confirms_timescaledb_extension() {
    let Some(maintenance_url) = bootstrap_test_url() else {
        eprintln!("SKIP database_bootstrap_integration: set DATABASE_BOOTSTRAP_TEST_URL");
        return;
    };

    let db_name = format!("bootstrap_ext_{}", Uuid::new_v4().simple());
    drop_database_if_exists(&maintenance_url, &db_name).await;

    let config = test_config(&db_name, &maintenance_url);
    ensure_database(&config).await.expect("bootstrap with extension");

    let app_url = config.app_database_maintenance_url();
    let app_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&app_url)
        .await
        .expect("app maintenance pool");
    let ext: Option<(String,)> =
        sqlx::query_as("SELECT extversion::text FROM pg_extension WHERE extname = 'timescaledb'")
            .fetch_optional(&app_pool)
            .await
            .expect("extension query");
    assert!(ext.is_some(), "timescaledb extension should be present");
    app_pool.close().await;

    drop_database_if_exists(&maintenance_url, &db_name).await;
}
