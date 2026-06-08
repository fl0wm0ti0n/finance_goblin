//! US-0018 — expense_series_by_month spine, uncategorized sentinel, catalog API.

use chrono::NaiveDate;
use flow_finance_ai::db::repositories;
use flow_finance_ai::transactions::repository::TransactionsRepository;
use flow_finance_ai::transactions::service::{expense_series_window, TransactionsService};
use flow_finance_ai::transactions::types::{
    ExpenseSeriesCategory, UNCATEGORIZED_CATEGORY_ID,
};
use flow_finance_ai::config::PrivacyConfig;
use serde_json::json;
use sqlx::PgPool;

fn skip_without_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async { PgPool::connect(&url).await.ok() })
}

async fn seed_us0018(pool: &PgPool) {
    repositories::upsert_category(pool, "cat-grocery", Some("Groceries"), &json!({}))
        .await
        .expect("grocery category");

    let dates = [
        (NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(), -50.0, Some("cat-grocery")),
        (NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(), -30.0, Some("cat-grocery")),
        (NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(), -20.0, None::<&str>),
    ];
    for (i, (date, amount, cat)) in dates.into_iter().enumerate() {
        repositories::upsert_transaction(
            pool,
            &format!("tx-us0018-{i}"),
            None,
            Some(date),
            Some(amount),
            Some("seed"),
            cat,
            &json!({}),
        )
        .await
        .expect("seed tx");
    }
}

#[tokio::test]
async fn expense_series_spine_includes_zero_months() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP expense_series_spine: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
    seed_us0018(&pool).await;

    let end = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let (start, end_bound) = expense_series_window(end, 6);
    let repo = TransactionsRepository::new(pool);
    let series = repo
        .expense_series_by_month(ExpenseSeriesCategory::MirrorId("cat-grocery"), start, end_bound)
        .await
        .unwrap();

    assert_eq!(series.len(), 6);
    assert!(series.iter().any(|m| m.month == "2026-01" && m.outflow_eur == 0.0));
    let april = series.iter().find(|m| m.month == "2026-04").unwrap();
    assert!((april.outflow_eur - 50.0).abs() < 0.01);
}

#[tokio::test]
async fn expense_series_uncategorized_bucket_metadata() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP uncategorized: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
    seed_us0018(&pool).await;

    let privacy = PrivacyConfig::default();
    let svc = TransactionsService::new(flow_finance_ai::db::DbPool::from_pool(pool), privacy);
    let resp = svc
        .expense_series(UNCATEGORIZED_CATEGORY_ID, 3, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.uncategorized, Some(true));
    assert_eq!(resp.category_label.as_deref(), Some("Uncategorized"));
    assert!(resp.transaction_count >= 1);
}

#[tokio::test]
async fn expense_series_unknown_category_returns_not_found() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP unknown category: DATABASE_URL not set");
        return;
    };
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");

    let privacy = PrivacyConfig::default();
    let svc = TransactionsService::new(flow_finance_ai::db::DbPool::from_pool(pool), privacy);
    let err = svc
        .expense_series("missing-cat", 12, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        flow_finance_ai::transactions::TransactionsError::NotFound(_)
    ));
}
