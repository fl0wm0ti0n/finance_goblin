use flow_finance_ai::config::SubscriptionsConfig;
use flow_finance_ai::subscriptions::repository::SubscriptionRepository;
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

async fn seed_recurring_transactions(pool: &PgPool, description: &str) -> String {
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
            VALUES ($1, $2, $3, -12.99, $4, $5)
            ON CONFLICT (firefly_id) DO UPDATE SET amount = EXCLUDED.amount, date = EXCLUDED.date
            "#,
        )
        .bind(format!("netflix-{month}-{description}"))
        .bind(&account_id)
        .bind(date)
        .bind(description)
        .bind(json!({"attributes": {"type": "withdrawal"}}))
        .execute(pool)
        .await
        .expect("seed netflix");
    }

    account_id
}

async fn insert_sync_run(pool: &PgPool) -> Uuid {
    let sync_run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(sync_run_id)
        .execute(pool)
        .await
        .expect("sync run");
    sync_run_id
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

    let _account = seed_recurring_transactions(&pool, "Netflix Streaming").await;
    let sync_run_id = insert_sync_run(&pool).await;

    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );

    service.run_detection(sync_run_id).await.expect("detection");

    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'pending'",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");

    assert!(pending_count >= 1, "expected at least one pending pattern");

    let alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE alert_type = 'new_detection' AND read_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count alerts");

    assert!(alerts >= 1, "expected new_detection alert");
}

#[tokio::test]
async fn upsert_alert_dedupes_unread_fingerprints() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let pattern_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO subscription_patterns (
            fingerprint, status, kind, payee_key, display_name, interval_days,
            current_amount, confidence_pct, first_seen_at, last_seen_at
        )
        VALUES ('dedup-test-fp', 'pending', 'subscription', 'netflix', 'Netflix', 30, 12.99, 95,
                CURRENT_DATE, CURRENT_DATE)
        RETURNING id
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("pattern");

    let repo = SubscriptionRepository::new(pool.clone(), test_subscriptions_config());
    let sync_run_id = insert_sync_run(&pool).await;
    let fp = SubscriptionRepository::compute_alert_fingerprint("new_detection", pattern_id, None, None, None);

    repo
        .upsert_alert(
            Some(pattern_id),
            "new_detection",
            "New recurring pattern: Netflix",
            Some("body"),
            sync_run_id,
            &fp,
        )
        .await
        .expect("first upsert");
    repo
        .upsert_alert(
            Some(pattern_id),
            "new_detection",
            "New recurring pattern: Netflix",
            Some("body updated"),
            sync_run_id,
            &fp,
        )
        .await
        .expect("second upsert");

    let unread: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE fingerprint = $1 AND read_at IS NULL",
    )
    .bind(&fp)
    .fetch_one(&pool)
    .await
    .expect("count");

    assert_eq!(unread, 1, "expected single unread row per fingerprint");
}

#[tokio::test]
async fn unchanged_resync_does_not_spam_alerts() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let _account = seed_recurring_transactions(&pool, "Netflix Resync Test").await;
    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );

    let sync1 = insert_sync_run(&pool).await;
    service.run_detection(sync1).await.expect("first detection");

    let after_first: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE alert_type = 'new_detection' AND read_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count");

    let sync2 = insert_sync_run(&pool).await;
    service.run_detection(sync2).await.expect("second detection");

    let after_second: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE alert_type = 'new_detection' AND read_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count");

    assert!(after_first >= 1);
    assert_eq!(
        after_first, after_second,
        "unchanged pending resync must not create duplicate unread alerts"
    );
}

#[tokio::test]
async fn unread_count_api_reconciled_semantics() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let _account = seed_recurring_transactions(&pool, "Reconcile Test").await;
    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );
    let sync_run_id = insert_sync_run(&pool).await;
    service.run_detection(sync_run_id).await.expect("detection");

    let counts = service.unread_alert_counts().await.expect("counts");
    assert!(counts.unread_new_detection <= counts.pending_patterns);
    assert!(counts.reconciled);
    assert!(counts.reconciliation_note.contains("price_change"));
}

#[tokio::test]
async fn confirm_marks_read_orphan_alerts() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let _account = seed_recurring_transactions(&pool, "Confirm Orphan Test").await;
    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );
    let sync_run_id = insert_sync_run(&pool).await;
    service.run_detection(sync_run_id).await.expect("detection");

    let pattern_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM subscription_patterns WHERE status = 'pending' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("pattern");

    let repo = SubscriptionRepository::new(pool.clone(), test_subscriptions_config());
    repo.confirm_pattern(pattern_id, None)
        .await
        .expect("confirm")
        .expect("confirmed row");

    let unread: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_alerts WHERE pattern_id = $1 AND read_at IS NULL",
    )
    .bind(pattern_id)
    .fetch_one(&pool)
    .await
    .expect("count");

    assert_eq!(unread, 0, "confirm should mark-read orphan alerts");
}

#[tokio::test]
async fn sepa_transfer_fixtures_merge_under_single_payee_key() {
    use flow_finance_ai::forecast::types::TransactionRow;
    use flow_finance_ai::recurrence::group::{by_payee, transaction_payee_key};
    use chrono::NaiveDate;

    let make_tx = |desc: &str, counterparty: &str| TransactionRow {
        firefly_id: format!("tx-{desc}"),
        account_id: None,
        date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        amount: -9.99,
        description: Some(desc.to_string()),
        category_id: None,
        payload: json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": counterparty,
                    "amount": "-9.99",
                    "type": "withdrawal"
                }]
            }
        }),
    };

    let tx1 = make_tx(
        "SVWZ+REF123456 UEBERWEISUNG Netflix Streaming",
        "Netflix",
    );
    let tx2 = make_tx(
        "SVWZ+REF789 UEBERWEISUNG Monatlich Netflix",
        "Netflix",
    );

    let key1 = transaction_payee_key(&tx1).expect("key1");
    let key2 = transaction_payee_key(&tx2).expect("key2");
    assert_eq!(key1, key2);
    assert_eq!(key1, "netflix");

    let txs = [tx1, tx2];
    let groups = by_payee(&txs);
    assert_eq!(groups.len(), 1);
    assert!(groups.contains_key("netflix"));
}

#[tokio::test]
async fn detection_window_defaults_to_730_days() {
    let config = test_subscriptions_config();
    assert_eq!(config.detection_window_days, 730);
}

#[tokio::test]
async fn forecast_recurring_still_detects_after_normalization() {
    use flow_finance_ai::forecast::ForecastService;
    use std::collections::HashMap;

    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account_id = seed_recurring_transactions(&pool, "Forecast Guard Netflix").await;
    let sync_run_id = insert_sync_run(&pool).await;
    let forecast_config = flow_finance_ai::config::ForecastConfig {
        rolling_window_days: 90,
        sparse_history_days: 90,
        retention_count: 5,
        recurring_amount_tolerance_pct: 5.0,
        category_buckets: HashMap::new(),
        ai_bucket_min_confidence: 0.75,
    };
    let forecast = ForecastService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        forecast_config,
    );

    let computation_id = forecast
        .recompute(sync_run_id, None)
        .await
        .expect("forecast recompute after normalization changes");

    let daily_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM forecast_balance_daily WHERE computation_id = $1 AND account_id = $2",
    )
    .bind(computation_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("count daily");

    assert!(daily_count > 0, "forecast recurring path must remain reachable");
}
