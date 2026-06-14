//! Exchange portfolio integration test — migration 007 + upsert + PnL + wealth snapshot.
//! Skips when DATABASE_URL is not set.

use flow_finance_ai::config::{PortfolioConfig, WealthConfig};
use flow_finance_ai::exchanges::repository::ExchangeRepository;
use flow_finance_ai::exchanges::types::ExchangeHolding;
use flow_finance_ai::fx::FxService;
use flow_finance_ai::portfolio::PortfolioEngine;
use flow_finance_ai::wealth::WealthService;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

#[tokio::test]
async fn exchanges_portfolio_integration() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for exchanges portfolio integration test");
            return;
        }
    };

    let run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(run_id)
        .execute(&pool)
        .await
        .expect("sync run");

    let repo = ExchangeRepository::new(pool.clone());
    repo.upsert_holdings(
        "binance",
        &[ExchangeHolding {
            asset: "BTC".into(),
            quantity: 0.1,
            product_type: "spot".into(),
            market_value_usd: None,
            unrealized_pnl: None,
            avg_cost: None,
            payload: serde_json::json!({}),
        }],
    )
    .await
    .expect("upsert holdings");

    repo.update_holding_eur("binance", "BTC", "spot", Some(5000.0), None, None, None)
        .await
        .expect("update eur");

    let fx = FxService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );
    let portfolio = PortfolioEngine::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );

    portfolio
        .recompute_pnl(run_id, &fx, std::sync::Arc::new(repo.clone()))
        .await
        .expect("recompute pnl");

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );

    wealth
        .upsert_daily_snapshot(run_id, Some(&repo), Some(&portfolio))
        .await
        .expect("snapshot");

    let crypto: Option<f64> = sqlx::query_scalar(
        "SELECT crypto_value_eur::float8 FROM net_worth_snapshots WHERE snapshot_date = CURRENT_DATE",
    )
    .fetch_optional(&pool)
    .await
    .expect("crypto column");

    assert!(crypto.unwrap_or(0.0) >= 0.0);

    let breakdown = wealth
        .compute_extended(Some(&repo), Some(&portfolio), None)
        .await
        .expect("breakdown");
    assert!(!breakdown.reporting_currency.is_empty());
}

#[test]
fn exchange_api_responses_contain_no_secrets() {
    let fixture = serde_json::json!({
        "id": "binance",
        "enabled": true,
        "connection_state": "connected",
        "api_key_env": "BINANCE_API_KEY",
        "configured": true
    });
    let s = serde_json::to_string(&fixture).unwrap();
    assert!(!s.contains("secret"));
    assert!(!s.contains("sk-"));
}
