//! BUG-0007 (DEC-0069) — category_search, mirror_date_bounds, subscriptions enrichment.

use chrono::NaiveDate;
use flow_finance_ai::ai::registry::{ToolRegistry, TOOL_NAMES};
use flow_finance_ai::config::PrivacyConfig;
use flow_finance_ai::db::repositories;
use flow_finance_ai::transactions::repository::TransactionsRepository;
use flow_finance_ai::transactions::{AggregateFilter, GroupBy, TransactionsService};
use serde_json::json;
use sqlx::PgPool;

fn skip_without_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async { PgPool::connect(&url).await.ok() })
}

async fn seed_categories(pool: &PgPool) {
    repositories::upsert_category(pool, "146", Some("Stromkosten"), &json!({}))
        .await
        .expect("strom category");
    repositories::upsert_category(pool, "47", Some("Amazon"), &json!({}))
        .await
        .expect("amazon category");
    for i in 0..12 {
        repositories::upsert_category(
            pool,
            &format!("cat-{i}"),
            Some(&format!("TestCategory{i:02}")),
            &json!({}),
        )
        .await
        .expect("overflow category");
    }
}

#[test]
fn six_tool_registry_unchanged() {
    let reg = ToolRegistry::build();
    assert_eq!(reg.tools().len(), 6);
    assert_eq!(TOOL_NAMES.len(), 6);
}

#[test]
fn get_transactions_schema_has_category_search() {
    let reg = ToolRegistry::build();
    let tool = reg.get("get_transactions").expect("tool");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["category_search"].is_object());
}

#[test]
fn get_subscriptions_schema_has_kind_enum() {
    let reg = ToolRegistry::build();
    let tool = reg.get("get_subscriptions").expect("tool");
    let schema = tool.parameters_schema();
    let kinds = schema["properties"]["kind"]["enum"].as_array().unwrap();
    assert!(kinds.contains(&json!("subscription")));
}

#[tokio::test]
async fn category_search_ilike_resolves_strom_and_amazon() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP category_search_ilike: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
    seed_categories(&pool).await;

    let repo = TransactionsRepository::new(pool.clone());
    let (strom, strom_trunc) = repo.search_categories_by_name("strom").await.unwrap();
    assert!(!strom_trunc);
    assert!(strom.iter().any(|m| m.category_id == "146"));

    let (amazon, _) = repo.search_categories_by_name("amazon").await.unwrap();
    assert!(amazon.iter().any(|m| m.category_id == "47"));
}

#[tokio::test]
async fn category_search_cap_ten_with_truncation_flag() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP category_search_cap: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
    seed_categories(&pool).await;

    let repo = TransactionsRepository::new(pool);
    let (matches, truncated) = repo.search_categories_by_name("TestCategory").await.unwrap();
    assert!(truncated);
    assert_eq!(matches.len(), 10);
}

#[tokio::test]
async fn mirror_date_bounds_from_transactions() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP mirror_date_bounds: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");

    let d1 = NaiveDate::from_ymd_opt(2025, 6, 5).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 5, 22).unwrap();
    repositories::upsert_transaction(
        &pool,
        "tx-bounds-1",
        None,
        Some(d1),
        Some(-10.0),
        None,
        Some("146"),
        &json!({}),
    )
    .await
    .unwrap();
    repositories::upsert_transaction(
        &pool,
        "tx-bounds-2",
        None,
        Some(d2),
        Some(-20.0),
        None,
        Some("47"),
        &json!({}),
    )
    .await
    .unwrap();

    let repo = TransactionsRepository::new(pool);
    let (min, max) = repo.mirror_date_bounds().await.unwrap();
    assert_eq!(min, Some(d1));
    assert_eq!(max, Some(d2));
}

#[tokio::test]
async fn aggregates_include_discovery_fields() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP aggregates_discovery: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
    seed_categories(&pool).await;

    let privacy = PrivacyConfig {
        allow_raw_transactions: false,
        ..Default::default()
    };
    let svc = TransactionsService::new(flow_finance_ai::db::DbPool::from_pool(pool), privacy);

    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let agg = svc
        .aggregates(AggregateFilter {
            period_start: start,
            period_end: end,
            category_id: None,
            category_search: Some("amazon".into()),
            group_by: GroupBy::Category,
        })
        .await
        .unwrap();

    assert!(agg.search_attempted);
    assert!(!agg.category_matches.is_empty());
    assert!(agg.mirror_date_bounds.min.is_some() || agg.mirror_date_bounds.max.is_some());
    assert!(agg.raw_rows.is_none());
}

#[tokio::test]
async fn category_search_short_keyword_rejected() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP short_keyword: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");

    let privacy = PrivacyConfig::default();
    let svc = TransactionsService::new(flow_finance_ai::db::DbPool::from_pool(pool), privacy);
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    let err = svc
        .aggregates(AggregateFilter {
            period_start: start,
            period_end: end,
            category_id: None,
            category_search: Some("a".into()),
            group_by: GroupBy::Category,
        })
        .await
        .unwrap_err();
    assert!(err.to_string().contains("category_search"));
}
