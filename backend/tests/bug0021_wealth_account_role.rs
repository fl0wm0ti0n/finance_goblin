//! BUG-0021 / DEC-0111 — wealth account_role COALESCE path (attributes + root fallback).

use flow_finance_ai::config::WealthConfig;
use flow_finance_ai::wealth::repository::WealthRepository;
use flow_finance_ai::wealth::WealthService;
use serde_json::json;
use sqlx::PgPool;

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_account(pool: &PgPool, firefly_id: &str, name: &str, payload: serde_json::Value) {
    sqlx::query(
        r#"
        INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
        VALUES ($1, 'asset', $2, 'EUR', 100.0, $3)
        ON CONFLICT (firefly_id) DO UPDATE SET
            name = EXCLUDED.name,
            payload = EXCLUDED.payload
        "#,
    )
    .bind(firefly_id)
    .bind(name)
    .bind(payload)
    .execute(pool)
    .await
    .expect("seed account");
}

#[tokio::test]
async fn nested_attributes_account_role_extracted() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0021 integration test");
            return;
        }
    };

    seed_account(
        &pool,
        "bug0021-nested-cash",
        "BUG0021 Cash",
        json!({
            "attributes": { "account_role": "cashWalletAsset" },
            "active": true,
            "include_net_worth": true
        }),
    )
    .await;
    seed_account(
        &pool,
        "bug0021-nested-savings",
        "BUG0021 Savings",
        json!({
            "attributes": { "account_role": "savingAsset" },
            "active": true,
            "include_net_worth": true
        }),
    )
    .await;
    seed_account(
        &pool,
        "bug0021-nested-checking",
        "BUG0021 Checking",
        json!({
            "attributes": { "account_role": "defaultAsset" },
            "active": true,
            "include_net_worth": true
        }),
    )
    .await;

    let repo = WealthRepository::new(pool.clone(), WealthConfig::default());
    let rows = repo.load_asset_accounts().await.expect("load accounts");

    let cash = rows
        .iter()
        .find(|r| r.firefly_id == "bug0021-nested-cash")
        .expect("cash account");
    assert_eq!(cash.account_role.as_deref(), Some("cashWalletAsset"));

    let savings = rows
        .iter()
        .find(|r| r.firefly_id == "bug0021-nested-savings")
        .expect("savings account");
    assert_eq!(savings.account_role.as_deref(), Some("savingAsset"));

    let checking = rows
        .iter()
        .find(|r| r.firefly_id == "bug0021-nested-checking")
        .expect("checking account");
    assert_eq!(checking.account_role.as_deref(), Some("defaultAsset"));
}

#[tokio::test]
async fn root_only_account_role_fallback() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0021 integration test");
            return;
        }
    };

    seed_account(
        &pool,
        "bug0021-root-only",
        "BUG0021 Root Role",
        json!({
            "account_role": "sharedAsset",
            "active": true,
            "include_net_worth": true
        }),
    )
    .await;

    let repo = WealthRepository::new(pool.clone(), WealthConfig::default());
    let rows = repo.load_asset_accounts().await.expect("load accounts");

    let row = rows
        .iter()
        .find(|r| r.firefly_id == "bug0021-root-only")
        .expect("root-only account");
    assert_eq!(row.account_role.as_deref(), Some("sharedAsset"));
}

#[tokio::test]
async fn wealth_api_shape_includes_account_role() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0021 integration test");
            return;
        }
    };

    seed_account(
        &pool,
        "bug0021-api-shape",
        "BUG0021 API Shape",
        json!({
            "attributes": { "account_role": "defaultAsset" },
            "active": true,
            "include_net_worth": true
        }),
    )
    .await;

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );

    let breakdown = wealth
        .compute_extended(None, None, None)
        .await
        .expect("compute extended");

    let row = breakdown
        .firefly
        .accounts
        .iter()
        .find(|a| a.firefly_id == "bug0021-api-shape")
        .expect("api shape account");

    assert_eq!(row.account_role.as_deref(), Some("defaultAsset"));
    assert!(!breakdown.firefly.accounts.is_empty());
    assert!(breakdown.total_eur >= 0.0);
}

#[tokio::test]
async fn categories_table_regression_smoke() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0021 integration test");
            return;
        }
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
        .fetch_one(&pool)
        .await
        .expect("categories count");

    assert!(count >= 0, "categories table must remain queryable");
}
