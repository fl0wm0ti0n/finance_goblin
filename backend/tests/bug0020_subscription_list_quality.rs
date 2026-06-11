//! BUG-0020 / DEC-0109 — subscription list reconcile, display_category backfill, forward guard.

use chrono::{NaiveDate, Utc};
use flow_finance_ai::config::SubscriptionsConfig;
use flow_finance_ai::recurrence::compute_fingerprint;
use flow_finance_ai::subscriptions::discovery::{run_discover, DiscoverQuery};
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

async fn rerun_migration_016(pool: &PgPool) {
    let sql = include_str!("../migrations/016_bug0020_subscription_list_quality.sql");
    sqlx::raw_sql(sql).execute(pool).await.expect("migration 016");
}

async fn seed_account(pool: &PgPool, account_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', 'BUG0020 Test', 'EUR', 1000.00, '{}')
        ON CONFLICT (firefly_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(pool)
    .await
    .expect("seed account");
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

async fn insert_pattern(
    pool: &PgPool,
    fingerprint: &str,
    status: &str,
    payee_key: &str,
    display_name: &str,
    interval_days: i32,
    amount: f64,
    confirmed_at: Option<chrono::DateTime<Utc>>,
) -> Uuid {
    let today = Utc::now().date_naive();
    sqlx::query_scalar(
        r#"
        INSERT INTO subscription_patterns (
            fingerprint, status, kind, payee_key, display_name, interval_days,
            current_amount, confidence_pct, first_seen_at, last_seen_at, confirmed_at
        )
        VALUES ($1, $2::subscription_status, 'subscription', $3, $4, $5, $6, 95, $7, $7, $8)
        RETURNING id
        "#,
    )
    .bind(fingerprint)
    .bind(status)
    .bind(payee_key)
    .bind(display_name)
    .bind(interval_days)
    .bind(amount)
    .bind(today)
    .bind(confirmed_at)
    .fetch_one(pool)
    .await
    .expect("insert pattern")
}

async fn link_tx(pool: &PgPool, pattern_id: Uuid, firefly_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO subscription_pattern_transactions (pattern_id, transaction_firefly_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(pattern_id)
    .bind(firefly_id)
    .execute(pool)
    .await
    .expect("link tx");
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
async fn bi_reconcile_merges_youtube_confirmed_cluster() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "bug0020-youtube-acct";
    seed_account(&pool, account).await;
    let today = Utc::now().date_naive();

    let survivor_fp = "bug0020-yt-survivor";
    let loser_fp = "bug0020-yt-loser";
    let display = "KAUF IRL IRELAND GOOGLE*YOUTUBEGOOGLE*YOUTUBE";

    let survivor_id = insert_pattern(
        &pool,
        survivor_fp,
        "confirmed",
        "kauf irl ireland google",
        display,
        30,
        -12.99,
        Some(Utc::now() - chrono::Duration::days(60)),
    )
    .await;
    let loser_id = insert_pattern(
        &pool,
        loser_fp,
        "confirmed",
        "kauf irl ireland google*youtubegoogle*youtube",
        display,
        30,
        -12.99,
        Some(Utc::now() - chrono::Duration::days(10)),
    )
    .await;

    seed_tx(
        &pool,
        "bug0020-yt-tx-1",
        account,
        today,
        -12.99,
        display,
        Some("66"),
    )
    .await;
    link_tx(&pool, survivor_id, "bug0020-yt-tx-1").await;
    link_tx(&pool, loser_id, "bug0020-yt-tx-1").await;

    rerun_migration_016(&pool).await;

    let dup_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM subscription_patterns
        WHERE status = 'confirmed' AND display_name = $1
        "#,
    )
    .bind(display)
    .fetch_one(&pool)
    .await
    .expect("count");

    assert_eq!(dup_count, 1, "YouTube cluster should collapse to one confirmed row");

    let loser_status: String = sqlx::query_scalar(
        "SELECT status::text FROM subscription_patterns WHERE id = $1",
    )
    .bind(loser_id)
    .fetch_one(&pool)
    .await
    .expect("loser status");

    assert_eq!(loser_status, "inactive");

    let payee_dup: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM (
          SELECT payee_key FROM subscription_patterns WHERE status = 'confirmed'
          GROUP BY payee_key HAVING COUNT(*) > 1
        ) d
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("payee dup");

    assert_eq!(payee_dup, 0, "no confirmed payee_key duplicates after reconcile");
}

#[tokio::test]
async fn bi_reconcile_collapses_strom_pending_cluster() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let display = "STROM BIRAGO G. 18/1";
    let pending_a = insert_pattern(
        &pool,
        "bug0020-strom-a",
        "pending",
        "220003055316 strom birago",
        display,
        31,
        -45.0,
        None,
    )
    .await;
    let pending_b = insert_pattern(
        &pool,
        "bug0020-strom-b",
        "pending",
        "strom birago g 18 1",
        display,
        31,
        -45.0,
        None,
    )
    .await;
    sqlx::query(
        "UPDATE subscription_patterns SET last_seen_at = CURRENT_DATE - 5 WHERE id = $1",
    )
    .bind(pending_a)
    .execute(&pool)
    .await
    .expect("age pending_a");
    let _rejected = insert_pattern(
        &pool,
        "bug0020-strom-rej",
        "rejected",
        "strom birago short",
        display,
        31,
        -45.0,
        None,
    )
    .await;

    rerun_migration_016(&pool).await;

    let pending_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM subscription_patterns
        WHERE status = 'pending' AND display_name = $1
        "#,
    )
    .bind(display)
    .fetch_one(&pool)
    .await
    .expect("pending count");

    assert_eq!(pending_count, 1, "Strom pending cluster should collapse to one survivor");

    let survivor_status: String = sqlx::query_scalar(
        "SELECT status::text FROM subscription_patterns WHERE id = $1",
    )
    .bind(pending_b)
    .fetch_one(&pool)
    .await
    .expect("survivor status");

    assert_eq!(
        survivor_status, "pending",
        "highest last_seen_at survivor stays pending"
    );

    let loser_status: String = sqlx::query_scalar(
        "SELECT status::text FROM subscription_patterns WHERE id = $1",
    )
    .bind(pending_a)
    .fetch_one(&pool)
    .await
    .expect("loser status");

    assert_eq!(loser_status, "rejected", "Strom pending loser marked rejected");

    let rejected_unchanged: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM subscription_patterns
        WHERE status = 'rejected' AND fingerprint = 'bug0020-strom-rej'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("rejected count");

    assert_eq!(rejected_unchanged, 1, "existing rejected Strom row unchanged");
}

#[tokio::test]
async fn bj_backfill_display_category_oracle() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "bug0020-bj-acct";
    seed_account(&pool, account).await;
    let today = Utc::now().date_naive();

    struct OracleRow {
        suffix: &'static str,
        payee_key: &'static str,
        display: &'static str,
        category: &'static str,
        tx_count: usize,
    }

    let rows = [
        OracleRow {
            suffix: "netflix",
            payee_key: "netflix",
            display: "Netflix",
            category: "18",
            tx_count: 3,
        },
        OracleRow {
            suffix: "kindle",
            payee_key: "kindle unltd",
            display: "Kindle Unltd",
            category: "18",
            tx_count: 2,
        },
        OracleRow {
            suffix: "youtube",
            payee_key: "kauf irl ireland google*youtubegoogle*youtube",
            display: "YouTube",
            category: "66",
            tx_count: 4,
        },
        OracleRow {
            suffix: "hgp",
            payee_key: "hgp unfall",
            display: "HGP Unfall",
            category: "56",
            tx_count: 3,
        },
        OracleRow {
            suffix: "florian",
            payee_key: "mitgliedsbeitrag - florian gabriel",
            display: "Mitgliedsbeitrag Florian Gabriel",
            category: "3",
            tx_count: 2,
        },
    ];

    for row in &rows {
        let fp = format!("bug0020-bj-{}", row.suffix);
        let pattern_id = insert_pattern(
            &pool,
            &fp,
            "confirmed",
            row.payee_key,
            row.display,
            30,
            -9.99,
            Some(Utc::now()),
        )
        .await;

        for i in 0..row.tx_count {
            let tx_id = format!("bug0020-bj-{}-{}", row.suffix, i);
            seed_tx(
                &pool,
                &tx_id,
                account,
                today - chrono::Duration::days(i as i64 * 30),
                -9.99,
                row.display,
                Some(row.category),
            )
            .await;
            link_tx(&pool, pattern_id, &tx_id).await;
        }
    }

    rerun_migration_016(&pool).await;

    let repo = SubscriptionRepository::new(pool.clone(), test_subscriptions_config());
    for row in &rows {
        let pattern_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM subscription_patterns WHERE fingerprint = $1",
        )
        .bind(format!("bug0020-bj-{}", row.suffix))
        .fetch_one(&pool)
        .await
        .expect("pattern id");

        let stored: Option<String> = sqlx::query_scalar(
            "SELECT display_category_id FROM subscription_patterns WHERE id = $1",
        )
        .bind(pattern_id)
        .fetch_one(&pool)
        .await
        .expect("stored category");

        let computed = repo
            .compute_display_category_id(pattern_id)
            .await
            .expect("compute category");

        assert_eq!(
            stored.as_deref(),
            Some(row.category),
            "{} display_category_id backfill",
            row.payee_key
        );
        assert_eq!(
            computed.as_deref(),
            Some(row.category),
            "{} compute_display_category_id matches backfill",
            row.payee_key
        );
    }
}

#[tokio::test]
async fn reg_discover_response_shape_unchanged() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "bug0020-discover-acct";
    seed_account(&pool, account).await;
    let today = Utc::now().date_naive();
    for month in 0..4 {
        seed_tx(
            &pool,
            &format!("bug0020-disc-{month}"),
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
    .expect("discover");

    assert!(response.meta.limit >= 1);
    assert!(!response.candidates.is_empty());
    let c = &response.candidates[0];
    assert!(!c.payee_key.is_empty());
    assert!(!c.display_name.is_empty());
    assert!(c.interval_days > 0);
    assert!(c.transaction_count >= 3);
    assert!(!c.transaction_ids.is_empty());
}

#[tokio::test]
async fn reg_tag_assign_and_filter_smoke() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let pattern_id = insert_pattern(
        &pool,
        "bug0020-tag-fp",
        "confirmed",
        "tag-test-merchant",
        "Tag Test Merchant",
        30,
        -5.0,
        Some(Utc::now()),
    )
    .await;

    let repo = SubscriptionRepository::new(pool.clone(), test_subscriptions_config());
    let tag = repo
        .create_operator_tag("BUG0020 Tag")
        .await
        .expect("create tag");
    repo.replace_pattern_tags(pattern_id, &[tag.id])
        .await
        .expect("assign tags");

    let filtered = repo
        .list_patterns(None, None, Some(&tag.slug))
        .await
        .expect("list by tag");

    assert!(
        filtered.iter().any(|p| p.id == pattern_id),
        "tag filter should return assigned pattern"
    );
}

#[tokio::test]
async fn da3_skips_pending_when_confirmed_merge_fingerprint_conflicts() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    let account = "bug0020-da3-acct";
    seed_account(&pool, account).await;
    let today = Utc::now().date_naive();
    let payee_key = "bug0020 merge guard merchant";
    let display = "BUG0020 Merge Guard Merchant";

    let fp_existing = compute_fingerprint(payee_key, 30, -10.00);
    let fp_conflict = compute_fingerprint(payee_key, 30, -15.00);

    insert_pattern(
        &pool,
        &fp_existing,
        "confirmed",
        payee_key,
        display,
        30,
        -10.0,
        Some(Utc::now()),
    )
    .await;
    insert_pattern(
        &pool,
        &fp_conflict,
        "confirmed",
        "other merchant key",
        "Other Merchant",
        30,
        -15.0,
        Some(Utc::now()),
    )
    .await;

    for month in 0..4 {
        seed_tx(
            &pool,
            &format!("bug0020-da3-{month}"),
            account,
            today - chrono::Duration::days(30 * month),
            -15.0,
            display,
            Some("18"),
        )
        .await;
    }

    let sync_run_id = insert_sync_run(&pool).await;
    let service = SubscriptionService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        test_subscriptions_config(),
    );
    service.run_detection(sync_run_id).await.expect("detection");

    let pending_for_payee: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM subscription_patterns
        WHERE status = 'pending' AND payee_key = $1
        "#,
    )
    .bind(payee_key)
    .fetch_one(&pool)
    .await
    .expect("pending count");

    assert_eq!(
        pending_for_payee, 0,
        "DA3 guard must not insert pending when confirmed merge fails fingerprint conflict"
    );
}

#[tokio::test]
async fn migration_016_is_idempotent() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set");
            return;
        }
    };

    rerun_migration_016(&pool).await;
    rerun_migration_016(&pool).await;

    let confirmed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM subscription_patterns WHERE status = 'confirmed'",
    )
    .fetch_one(&pool)
    .await
    .expect("confirmed count");

    assert!(confirmed >= 0, "second migration 016 run must not error");
}
