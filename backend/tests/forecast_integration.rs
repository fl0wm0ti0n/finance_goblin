use std::collections::HashMap;

use flow_finance_ai::config::ForecastConfig;
use flow_finance_ai::forecast::repository::ForecastRepository;
use flow_finance_ai::forecast::ForecastService;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn test_forecast_config() -> ForecastConfig {
    ForecastConfig {
        rolling_window_days: 90,
        sparse_history_days: 90,
        retention_count: 5,
        recurring_amount_tolerance_pct: 5.0,
        category_buckets: HashMap::new(),
        ai_bucket_min_confidence: 0.75,
    }
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_fixture(pool: &PgPool) -> String {
    let account_id = "test-checking-1".to_string();
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'Test Checking', 'EUR', 1500.00, '{}')
        ON CONFLICT (firefly_id) DO UPDATE SET balance = EXCLUDED.balance
        "#,
    )
    .bind(&account_id)
    .execute(pool)
    .await
    .expect("seed account");

    let today = chrono::Utc::now().date_naive();
    for month in 0..4 {
        let date = today - chrono::Duration::days(30 * month);
        sqlx::query(
            r#"
            INSERT INTO transactions (firefly_id, account_id, date, amount, description, payload)
            VALUES ($1, $2, $3, 3000.0, 'Salary', $4)
            ON CONFLICT (firefly_id) DO UPDATE SET amount = EXCLUDED.amount
            "#,
        )
        .bind(format!("salary-{month}"))
        .bind(&account_id)
        .bind(date)
        .bind(json!({"attributes": {"type": "deposit"}}))
        .execute(pool)
        .await
        .expect("seed tx");
    }

    for day in 1..=10 {
        let date = today - chrono::Duration::days(day);
        sqlx::query(
            r#"
            INSERT INTO transactions (firefly_id, account_id, date, amount, description, payload)
            VALUES ($1, $2, $3, -25.0, 'Coffee', $4)
            ON CONFLICT (firefly_id) DO NOTHING
            "#,
        )
        .bind(format!("coffee-{day}"))
        .bind(&account_id)
        .bind(date)
        .bind(json!({"attributes": {"type": "withdrawal"}}))
        .execute(pool)
        .await
        .expect("seed spend");
    }

    account_id
}

#[tokio::test]
async fn forecast_recompute_persists_hypertable_rows() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for forecast integration test");
            return;
        }
    };

    let account_id = seed_fixture(&pool).await;
    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(&pool)
        .await
        .expect("sync run");

    let service = ForecastService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_forecast_config(),
    );

    let computation_id = service
        .recompute(sync_run_id, None)
        .await
        .expect("recompute should succeed");

    let daily_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM forecast_balance_daily WHERE computation_id = $1 AND account_id = $2",
    )
    .bind(computation_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("count daily");

    assert!(daily_count > 30, "expected daily forecast rows");

    let monthly_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM forecast_cashflow_monthly WHERE computation_id = $1 AND account_id = $2",
    )
    .bind(computation_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("count monthly");

    assert!(monthly_count > 0, "expected monthly forecast rows");
}

#[tokio::test]
async fn forecast_meta_stale_when_latest_failed() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for forecast integration test");
            return;
        }
    };

    let _ = seed_fixture(&pool).await;
    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(&pool)
        .await
        .expect("sync run");

    let service = ForecastService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_forecast_config(),
    );
    service.recompute(sync_run_id, None).await.expect("first recompute");

    let failed_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO forecast_computations (id, sync_run_id, status, error_message) VALUES ($1, $2, 'failed', 'simulated')",
    )
    .bind(failed_id)
    .bind(sync_run_id)
    .execute(&pool)
    .await
    .expect("failed row");

    let repo = service.repository();
    let success = repo.latest_successful().await.expect("success");
    let any = repo.latest_any().await.expect("any");
    assert!(repo.is_stale(&success, &any));
}

#[tokio::test]
async fn forecast_retention_prunes_paired_ml_without_fk_violation() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for forecast retention integration test");
            return;
        }
    };

    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(&pool)
        .await
        .expect("sync run");

    let mut config = test_forecast_config();
    config.retention_count = 2;
    let repo = ForecastRepository::new(pool.clone(), config);

    for i in 0..4 {
        let baseline_id = Uuid::new_v4();
        let ml_id = Uuid::new_v4();
        let hours_ago = (4 - i) * 24;

        sqlx::query(
            r#"
            INSERT INTO forecast_computations (id, sync_run_id, status, model_kind, computed_at)
            VALUES ($1, $2, 'success', 'baseline', NOW() - ($3::text || ' hours')::interval)
            "#,
        )
        .bind(baseline_id)
        .bind(sync_run_id)
        .bind(hours_ago)
        .execute(&pool)
        .await
        .expect("seed baseline");

        sqlx::query(
            r#"
            INSERT INTO forecast_computations
              (id, sync_run_id, status, model_kind, paired_baseline_id, computed_at)
            VALUES ($1, $2, 'success', 'ml_enhanced', $3, NOW() - ($4::text || ' hours')::interval)
            "#,
        )
        .bind(ml_id)
        .bind(sync_run_id)
        .bind(baseline_id)
        .bind(hours_ago)
        .execute(&pool)
        .await
        .expect("seed ml_enhanced");
    }

    repo.enforce_retention()
        .await
        .expect("retention should succeed without FK violation");

    let baseline_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM forecast_computations WHERE status = 'success' AND model_kind = 'baseline'",
    )
    .fetch_one(&pool)
    .await
    .expect("count baseline");

    let ml_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM forecast_computations WHERE status = 'success' AND model_kind = 'ml_enhanced'",
    )
    .fetch_one(&pool)
    .await
    .expect("count ml");

    assert_eq!(baseline_count, 2, "expected two baseline rows after retention");
    assert_eq!(ml_count, 2, "expected two ml_enhanced rows after retention");
}
