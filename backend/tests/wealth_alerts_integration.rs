//! Wealth and alerts integration test — snapshot upsert + scarcity alert path.
//! Skips when DATABASE_URL is not set.

use flow_finance_ai::alerts::{AlertService, EvalContext};
use flow_finance_ai::config::{AlertsConfig, WealthConfig};
use flow_finance_ai::wealth::WealthService;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_scarcity_fixture(pool: &PgPool) -> (Uuid, Uuid) {
    let account_id = "wealth-test-account".to_string();
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'Wealth Test', 'EUR', 150.0, '{"include_net_worth": true}')
        ON CONFLICT (firefly_id) DO UPDATE SET balance = EXCLUDED.balance
        "#,
    )
    .bind(&account_id)
    .execute(pool)
    .await
    .expect("seed account");

    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(pool)
        .await
        .expect("sync run");

    let forecast_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO forecast_computations (id, sync_run_id, status) VALUES ($1, $2, 'success')",
    )
    .bind(forecast_id)
    .bind(sync_run_id)
    .execute(pool)
    .await
    .expect("forecast computation");

    let today = chrono::Utc::now().date_naive();
    for offset in 0..46i64 {
        let date = today + chrono::Duration::days(offset);
        let ts = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let balance = if offset < 10 { 500.0 } else { 150.0 };
        sqlx::query(
            "INSERT INTO forecast_balance_daily (ts, account_id, computation_id, balance) VALUES ($1, $2, $3, $4)",
        )
        .bind(ts)
        .bind(&account_id)
        .bind(forecast_id)
        .bind(balance)
        .execute(pool)
        .await
        .expect("forecast daily");
    }

    (sync_run_id, forecast_id)
}

#[tokio::test]
async fn wealth_snapshot_and_scarcity_alert_on_post_sync() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for wealth/alerts integration test");
            return;
        }
    };

    let (run_id, forecast_id) = seed_scarcity_fixture(&pool).await;

    let alerts_config = AlertsConfig {
        scarcity_threshold_eur: 200.0,
        budget_drift_pct: 20.0,
        reporting_currency: "EUR".into(),
    };

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig {
            snapshot_retention_days: 365,
        },
        "EUR".into(),
    );

    let alerts = AlertService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        alerts_config.clone(),
        wealth.clone(),
        None,
    );

    let breakdown = wealth
        .compute_breakdown(None, None)
        .await
        .expect("breakdown");
    assert!(breakdown.total >= 150.0);

    wealth
        .upsert_daily_snapshot(run_id, None, None)
        .await
        .expect("snapshot");

    let snapshot_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM net_worth_snapshots WHERE snapshot_date = CURRENT_DATE",
    )
    .fetch_one(&pool)
    .await
    .expect("snapshot count");
    assert_eq!(snapshot_count, 1);

    let eval_ctx = EvalContext {
        forecast_computation_id: forecast_id,
        plan_computation_id: None,
        config: alerts_config,
    };

    let result = alerts.run_post_sync(run_id, eval_ctx).await.expect("eval");
    assert!(result.created >= 1 || result.updated >= 1);

    let alert_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE alert_type = 'scarcity' AND status = 'active'",
    )
    .fetch_one(&pool)
    .await
    .expect("alert count");
    assert!(alert_count >= 1);
}

#[test]
fn wealth_alerts_modules_have_no_firefly_writes() {
    let wealth_src = include_str!("../src/wealth/service.rs");
    let alerts_src = include_str!("../src/alerts/service.rs");
    let evaluate_src = include_str!("../src/alerts/evaluate.rs");

    for (name, src) in [
        ("wealth/service", wealth_src),
        ("alerts/service", alerts_src),
        ("alerts/evaluate", evaluate_src),
    ] {
        assert!(
            !src.contains("FireflyClient"),
            "{name} must not use FireflyClient"
        );
        assert!(
            !src.contains("firefly::"),
            "{name} must not import firefly module"
        );
    }
}

#[test]
fn cashflow_dashboard_uses_scarcity_threshold_variable() {
    let json = include_str!("../../grafana/provisioning/dashboards/analytics/cashflow.json");
    assert!(json.contains("$scarcity_threshold"));
    assert!(json.contains("alert_config"));
}
