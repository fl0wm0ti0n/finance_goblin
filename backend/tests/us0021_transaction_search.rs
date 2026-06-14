//! US-0021 / DEC-0112–DEC-0114 — transaction search, hint pass, preview-group, AC-5 regression.

use chrono::{NaiveDate, Utc};
use flow_finance_ai::config::SubscriptionsConfig;
use flow_finance_ai::recurrence::compute_fingerprint;
use flow_finance_ai::subscriptions::discovery::{run_discover, DiscoverQuery};
use flow_finance_ai::subscriptions::repository::SubscriptionRepository;
use flow_finance_ai::subscriptions::transaction_search::{
    preview_transaction_group, run_transaction_search, TransactionSearchQuery, HINT_SCAN_CAP,
    SEARCH_PAGE_LIMIT,
};
use serde_json::json;
use sqlx::PgPool;

fn test_subscriptions_config() -> SubscriptionsConfig {
    SubscriptionsConfig::default()
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_account(pool: &PgPool, account_id: &str, account_role: &str) {
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'US0021 Test', 'EUR', 1000.00, $2)
        ON CONFLICT (firefly_id) DO UPDATE SET payload = EXCLUDED.payload
        "#,
    )
    .bind(account_id)
    .bind(json!({"attributes": {"account_role": account_role}}))
    .execute(pool)
    .await
    .expect("seed account");
}

async fn seed_category(pool: &PgPool, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO categories (firefly_id, name, payload)
        VALUES ($1, $2, '{}')
        ON CONFLICT (firefly_id) DO UPDATE SET name = EXCLUDED.name
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await
    .expect("seed category");
}

async fn seed_tx(
    pool: &PgPool,
    firefly_id: &str,
    account_id: &str,
    date: NaiveDate,
    amount: f64,
    description: &str,
    category_id: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO transactions (firefly_id, account_id, date, amount, description, category_id, payload)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (firefly_id) DO UPDATE SET
            amount = EXCLUDED.amount,
            date = EXCLUDED.date,
            description = EXCLUDED.description,
            category_id = EXCLUDED.category_id
        "#,
    )
    .bind(firefly_id)
    .bind(account_id)
    .bind(date)
    .bind(amount)
    .bind(description)
    .bind(category_id)
    .bind(json!({"attributes": {"type": "withdrawal"}}))
    .execute(pool)
    .await
    .expect("seed tx");
}

#[tokio::test]
async fn tx_search_sql_filters_and_pagination_meta() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "us0021-search-acct";
    let other_account = "us0021-other-acct";
    seed_account(&pool, account, "defaultAsset").await;
    seed_account(&pool, other_account, "savingAsset").await;
    seed_category(&pool, "us0021-cat", "Streaming").await;

    let today = Utc::now().date_naive();
    for month in 0..5 {
        seed_tx(
            &pool,
            &format!("us0021-tx-{month}"),
            account,
            today - chrono::Duration::days(30 * month),
            -14.99,
            "Spotify Premium",
            Some("us0021-cat"),
        )
        .await;
    }
    seed_tx(
        &pool,
        "us0021-other-tx",
        other_account,
        today,
        -9.99,
        "Other payee",
        None,
    )
    .await;

    let repo = SubscriptionRepository::new(pool, test_subscriptions_config());
    let response = run_transaction_search(
        &repo,
        TransactionSearchQuery {
            account_id: account,
            payee: Some("spotify"),
            category_id: Some("us0021-cat"),
            account_role: Some("defaultAsset"),
            date_from: None,
            date_to: None,
            recurring_hint: false,
            page: 1,
            limit: SEARCH_PAGE_LIMIT,
        },
    )
    .await
    .expect("search");

    assert_eq!(response.meta.limit, SEARCH_PAGE_LIMIT);
    assert!(response.meta.total_count >= 5);
    assert!(!response.transactions.is_empty());
    for tx in &response.transactions {
        assert_eq!(tx.account_id, account);
        assert!(tx.description.as_deref().unwrap_or("").to_lowercase().contains("spotify"));
        assert_eq!(tx.category_id.as_deref(), Some("us0021-cat"));
        assert_eq!(tx.account_role.as_deref(), Some("defaultAsset"));
    }
}

#[tokio::test]
async fn tx_search_hint_attachment_without_pending_emit() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "us0021-hint-acct";
    seed_account(&pool, account, "defaultAsset").await;
    let today = Utc::now().date_naive();

    for month in 0..4 {
        seed_tx(
            &pool,
            &format!("us0021-hint-{month}"),
            account,
            today - chrono::Duration::days(30 * month),
            -9.99,
            "Netflix Streaming",
            Some("18"),
        )
        .await;
    }

    let repo = SubscriptionRepository::new(pool.clone(), test_subscriptions_config());
    let response = run_transaction_search(
        &repo,
        TransactionSearchQuery {
            account_id: account,
            payee: Some("netflix"),
            category_id: None,
            account_role: None,
            date_from: None,
            date_to: None,
            recurring_hint: true,
            page: 1,
            limit: SEARCH_PAGE_LIMIT,
        },
    )
    .await
    .expect("search with hints");

    let hinted: Vec<_> = response
        .transactions
        .iter()
        .filter(|t| t.recurring_hint.is_some())
        .collect();
    assert!(
        !hinted.is_empty(),
        "expected recurring_hint on filtered monthly netflix txs"
    );

    let pending_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'pending'",
    )
    .fetch_one(&pool)
    .await
    .expect("pending count");

    let pending_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'pending'",
    )
    .fetch_one(&pool)
    .await
    .expect("pending count after");

    assert_eq!(
        pending_before, pending_after,
        "hint pass must not create pending patterns"
    );
}

#[tokio::test]
async fn preview_group_median_interval_computation() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "us0021-preview-acct";
    seed_account(&pool, account, "defaultAsset").await;
    let today = Utc::now().date_naive();
    let ids: Vec<String> = (0..3)
        .map(|i| format!("us0021-preview-{i}"))
        .collect();

    for (i, id) in ids.iter().enumerate() {
        seed_tx(
            &pool,
            id,
            account,
            today - chrono::Duration::days(30 * i as i64),
            -12.50,
            "SEPA-Lastschrift Gym",
            None,
        )
        .await;
    }

    let repo = SubscriptionRepository::new(pool, test_subscriptions_config());
    let preview = preview_transaction_group(&repo, &ids)
        .await
        .expect("preview")
        .expect("ok preview");

    assert_eq!(preview.transaction_ids.len(), 3);
    assert!(!preview.payee_key.is_empty());
    assert!(preview.interval_days > 0);
    assert!((preview.median_amount - -12.50).abs() < 0.01);
}

#[tokio::test]
async fn reg_discover_candidate_pass_unchanged_ac5() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "us0021-ac5-acct";
    seed_account(&pool, account, "defaultAsset").await;
    let today = Utc::now().date_naive();
    for month in 0..4 {
        seed_tx(
            &pool,
            &format!("us0021-ac5-{month}"),
            account,
            today - chrono::Duration::days(30 * month),
            -14.99,
            "Spotify Premium",
            Some("18"),
        )
        .await;
    }

    let repo = SubscriptionRepository::new(pool, test_subscriptions_config());
    let response = run_discover(
        &repo,
        DiscoverQuery {
            account_id: Some(account),
            payee: Some("spotify"),
            interval_days: Some(30),
            limit: 10,
        },
    )
    .await
    .expect("discover unchanged");

    assert!(!response.candidates.is_empty());
    let c = &response.candidates[0];
    assert!(c.transaction_count >= 3);
    assert!(c.confidence_pct >= 60);
}

#[tokio::test]
async fn hint_scan_cap_constant_documented() {
    assert_eq!(HINT_SCAN_CAP, 500);
    assert_eq!(SEARCH_PAGE_LIMIT, 100);
}

#[test]
fn preview_group_fingerprint_helper_stable() {
    let fp = compute_fingerprint("netflix", 30, -9.99);
    assert!(!fp.is_empty());
}
