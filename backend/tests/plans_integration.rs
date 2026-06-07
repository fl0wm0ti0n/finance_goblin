//! Plan integration test — create plan, apply template, plan-vs-actual path.
//! Skips when DATABASE_URL is not set.

use flow_finance_ai::config::{ForecastConfig, PlansConfig};
use flow_finance_ai::forecast::ForecastService;
use flow_finance_ai::plan::PlanService;
use flow_finance_ai::plan::types::PlanTemplate;
use flow_finance_ai::plan::templates::TemplateOverrides;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_forecast_baseline(pool: &PgPool) -> Uuid {
    let account_id = "plan-test-account".to_string();
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'Plan Test', 'EUR', 5000.00, '{}')
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
    for offset in 0..3i64 {
        let date = today + chrono::Duration::days(offset);
        let ts = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let balance = 5000.0 + offset as f64 * 10.0;
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

    forecast_id
}

#[tokio::test]
async fn plan_create_apply_recompute_plan_vs_actual() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for plan integration test");
            return;
        }
    };

    let _forecast_id = seed_forecast_baseline(&pool).await;
    let db = flow_finance_ai::db::DbPool::from_pool(pool.clone());
    let forecast = ForecastService::new(
        db.clone(),
        ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        },
    );
    let plans = PlanService::new(db, PlansConfig::default(), forecast);

    let (plan, version) = plans
        .create_plan("Integration Leasing", Some("leasing"))
        .await
        .expect("create plan");

    plans.activate_plan(plan.id).await.expect("activate");
    plans
        .apply_template(version.id, PlanTemplate::Leasing.as_str(), TemplateOverrides::default())
        .await
        .expect("apply template");

    // Allow async recompute to finish
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let _ = plans.recompute_with_latest_forecast(version.id).await;

    let pva = plans.plan_vs_actual(None).await.expect("plan vs actual");
    assert!(!pva.rows.is_empty() || pva.plan_stale);
    assert_eq!(pva.reporting_currency, "EUR");
}

#[tokio::test]
async fn compare_zero_adjustments_overlay_delta_is_zero() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for plan integration test");
            return;
        }
    };

    let _forecast_id = seed_forecast_baseline(&pool).await;
    let db = flow_finance_ai::db::DbPool::from_pool(pool.clone());
    let forecast = ForecastService::new(
        db.clone(),
        ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        },
    );
    let plans = PlanService::new(db, PlansConfig::default(), forecast);

    let (plan, version) = plans
        .create_plan("Empty Compare", Some("custom"))
        .await
        .expect("create plan");

    let _ = plans.recompute_with_latest_forecast(version.id).await;

    let compare = plans.compare_versions(plan.id).await.expect("compare");
    assert_eq!(
        compare.versions[0].monthly_delta_sum, "0.00",
        "empty adjustments must yield overlay-only zero delta"
    );
}

#[tokio::test]
async fn compare_leasing_template_overlay_delta_approx_minus_300() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for plan integration test");
            return;
        }
    };

    let _forecast_id = seed_forecast_baseline(&pool).await;
    let db = flow_finance_ai::db::DbPool::from_pool(pool.clone());
    let forecast = ForecastService::new(
        db.clone(),
        ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        },
    );
    let plans = PlanService::new(db, PlansConfig::default(), forecast);

    let (plan, version) = plans
        .create_plan("Leasing Compare", Some("leasing"))
        .await
        .expect("create plan");

    plans
        .apply_template(version.id, PlanTemplate::Leasing.as_str(), TemplateOverrides::default())
        .await
        .expect("apply template");

    let _ = plans.recompute_with_latest_forecast(version.id).await;

    let compare = plans.compare_versions(plan.id).await.expect("compare");
    let delta: f64 = compare.versions[0].monthly_delta_sum.parse().expect("parse delta");
    assert!(
        delta < -299.0 && delta > -301.0,
        "leasing overlay delta expected ~-300, got {delta}"
    );
}

#[tokio::test]
async fn plan_vs_actual_without_active_plan_returns_error() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for plan integration test");
            return;
        }
    };

    let _forecast_id = seed_forecast_baseline(&pool).await;
    let db = flow_finance_ai::db::DbPool::from_pool(pool.clone());
    let forecast = ForecastService::new(
        db.clone(),
        ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: std::collections::HashMap::new(),
            ai_bucket_min_confidence: 0.75,
        },
    );
    let plans = PlanService::new(db, PlansConfig::default(), forecast);

    let (_plan, _version) = plans
        .create_plan("Inactive PVA", Some("custom"))
        .await
        .expect("create plan");

    let err = plans
        .plan_vs_actual(None)
        .await
        .expect_err("service still errors without active plan");
    assert!(matches!(
        err,
        flow_finance_ai::plan::service::PlanError::NoActivePlan
    ));
}

#[test]
fn plan_module_has_no_firefly_writes() {
    let plan_dir = std::path::Path::new("src/plan");
    for entry in std::fs::read_dir(plan_dir).expect("plan dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read plan source");
        assert!(
            !content.contains("FireflyClient"),
            "plan module must not use FireflyClient: {}",
            path.display()
        );
        assert!(
            !content.contains("firefly::sync"),
            "plan module must not call firefly sync: {}",
            path.display()
        );
    }
}
