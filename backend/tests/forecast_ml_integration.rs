//! US-0009 forecast ML integration tests — require DATABASE_URL.

use flow_finance_ai::config::{ForecastConfig, ForecastMlConfig};
use flow_finance_ai::forecast::ForecastService;
use flow_finance_ai::forecast_ml::service::ForecastMlError;
use flow_finance_ai::forecast_ml::sidecar::{ForecastRequest, SidecarClient, SidecarPoint};
use flow_finance_ai::forecast_ml::ForecastMlService;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_forecast_config() -> ForecastConfig {
    ForecastConfig {
        rolling_window_days: 90,
        sparse_history_days: 90,
        retention_count: 5,
        recurring_amount_tolerance_pct: 5.0,
        category_buckets: HashMap::new(),
    }
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

fn sidecar_mock_body() -> serde_json::Value {
    json!({
        "model_family": "AutoETS",
        "seasonal_periods": [12],
        "seasonal_strength": 0.42,
        "seasonal_detected": true,
        "forecasts": (0..24).map(|i| {
            json!({
                "ds": format!("2025-{:02}-01", (i % 12) + 1),
                "y": 1200.0 + i as f64 * 10.0,
                "y_lo": 1000.0 + i as f64 * 8.0,
                "y_hi": 1400.0 + i as f64 * 12.0
            })
        }).collect::<Vec<_>>(),
        "backtest_wmape": 0.18,
        "low_confidence": false
    })
}

async fn seed_ml_fixture(pool: &PgPool) -> String {
    let account_id = format!("ml-test-{}", Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'ML Test', 'EUR', 5000.00, '{}')
        ON CONFLICT (firefly_id) DO UPDATE SET balance = EXCLUDED.balance
        "#,
    )
    .bind(&account_id)
    .execute(pool)
    .await
    .expect("seed account");

    let today = chrono::Utc::now().date_naive();
    for month in 0..14 {
        let date = today - chrono::Duration::days(30 * month);
        sqlx::query(
            r#"
            INSERT INTO transactions (firefly_id, account_id, date, amount, description, payload)
            VALUES ($1, $2, $3, 2500.0, 'Salary', $4)
            ON CONFLICT (firefly_id) DO NOTHING
            "#,
        )
        .bind(format!("ml-salary-{month}-{account_id}"))
        .bind(&account_id)
        .bind(date)
        .bind(json!({}))
        .execute(pool)
        .await
        .expect("seed salary");
    }
    account_id
}

#[tokio::test]
async fn sidecar_client_mock_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/forecast"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sidecar_mock_body()))
        .mount(&server)
        .await;

    let client = SidecarClient::new(&ForecastMlConfig {
        sidecar_url: server.uri(),
        sidecar_timeout_secs: 5,
        ..Default::default()
    });

    let resp = client
        .forecast(&ForecastRequest {
            series_id: "test".into(),
            freq: "MS".into(),
            points: (0..14)
                .map(|i| SidecarPoint {
                    ds: format!("2024-{:02}-01", (i % 12) + 1),
                    y: 1000.0 + i as f64,
                })
                .collect(),
            horizon: 6,
            level: vec![90],
            model: "auto".into(),
        })
        .await
        .expect("forecast ok");
    assert_eq!(resp.model_family, "AutoETS");
}

#[tokio::test]
async fn ml_skip_records_metadata_not_sync_failure() {
    let Some(pool) = setup_db().await else {
        eprintln!("SKIP ml_skip_records_metadata (no DATABASE_URL)");
        return;
    };

    let _account_id = seed_ml_fixture(&pool).await;
    let db = flow_finance_ai::db::DbPool::from_pool(pool.clone());
    let forecast = ForecastService::new(db, test_forecast_config());
    let repo = forecast.repository_arc();

    let run_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'running', 'test')",
    )
    .bind(run_id)
    .execute(&pool)
    .await
    .expect("sync run");

    let baseline_id = forecast.recompute(run_id, None).await.expect("baseline");

    let ml = ForecastMlService::new(
        repo.clone(),
        ForecastMlConfig {
            enabled: true,
            sidecar_url: "http://127.0.0.1:1".into(),
            sidecar_timeout_secs: 1,
            ..Default::default()
        },
    );

    let result = ml.recompute(run_id, baseline_id).await;
    assert!(result.is_err());

    ml.record_skip_on_baseline(baseline_id, &ForecastMlError::SidecarUnavailable)
        .await
        .expect("record skip");

    let row = repo
        .latest_successful_by_kind("baseline")
        .await
        .expect("query")
        .expect("baseline row");
    assert_eq!(
        row.metadata.get("ml_skipped_reason").and_then(|v| v.as_str()),
        Some("sidecar_unavailable")
    );
}

#[test]
fn grafana_dashboard_has_forecast_variant() {
    let json = include_str!("../../grafana/provisioning/dashboards/analytics/forecast-horizons.json");
    assert!(json.contains("$forecast_variant"));
    assert!(json.contains("ml_enhanced"));
    assert!(json.contains("\"uid\": \"forecast-horizons\""));
}
