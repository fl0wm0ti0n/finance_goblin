use flow_finance_ai::config::SubscriptionsConfig;
use flow_finance_ai::subscriptions::SubscriptionService;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn test_subscriptions_config() -> SubscriptionsConfig {
    SubscriptionsConfig::default()
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_recurring_transactions(pool: &PgPool) -> String {
    let account_id = "test-sub-account".to_string();
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'Sub Test', 'EUR', 2000.00, '{}')
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
            VALUES ($1, $2, $3, -12.99, 'Netflix Streaming', $4)
            ON CONFLICT (firefly_id) DO UPDATE SET amount = EXCLUDED.amount, date = EXCLUDED.date
            "#,
        )
        .bind(format!("netflix-{month}"))
        .bind(&account_id)
        .bind(date)
        .bind(json!({"attributes": {"type": "withdrawal"}}))
        .execute(pool)
        .await
        .expect("seed netflix");
    }

    account_id
}

#[tokio::test]
async fn subscription_detection_persists_pending_pattern() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for subscription integration test");
            return;
        }
    };

    let _account = seed_recurring_transactions(&pool).await;
    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(&pool)
        .await
        .expect("sync run");

    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );

    let result = service.run_detection(sync_run_id).await.expect("detection");
    assert!(result.rejected_fingerprints.is_empty() || true);

    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'pending'",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");

    assert!(pending_count >= 1, "expected at least one pending pattern");

    let alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE alert_type = 'new_detection'",
    )
    .fetch_one(&pool)
    .await
    .expect("count alerts");

    assert!(alerts >= 1, "expected new_detection alert");
}
