//! BUG-0023 / DEC-0064 / DEC-0080 / DEC-0081 / DEC-0038 — crypto wealth EUR regression.
//! Skips when DATABASE_URL is not set.

use chrono::Utc;
use flow_finance_ai::config::{PortfolioConfig, WealthConfig};
use flow_finance_ai::exchanges::repository::ExchangeRepository;
use flow_finance_ai::exchanges::types::ExchangeHolding;
use flow_finance_ai::fx::FxService;
use flow_finance_ai::portfolio::PortfolioEngine;
use flow_finance_ai::wealth::WealthService;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

async fn seed_fx_rate(pool: &PgPool) {
    let today = Utc::now().date_naive();
    sqlx::query(
        r#"
        INSERT INTO fx_rates (rate_date, base, quote, rate, provider)
        VALUES ($1, 'USD', 'EUR', 0.92, 'test')
        ON CONFLICT (rate_date, base, quote) DO UPDATE SET rate = EXCLUDED.rate
        "#,
    )
    .bind(today)
    .execute(pool)
    .await
    .expect("seed fx rate");
}

async fn seed_sync_run(pool: &PgPool) -> Uuid {
    let run_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sync_runs (id, status, trigger) VALUES ($1, 'success', 'test')")
        .bind(run_id)
        .execute(pool)
        .await
        .expect("sync run");
    run_id
}

async fn enable_bitunix(pool: &PgPool) {
    sqlx::query(
        "UPDATE exchange_connections SET enabled = true, connection_state = 'connected' WHERE id = 'bitunix'",
    )
    .execute(pool)
    .await
    .expect("enable bitunix");
}

async fn clear_bitunix_holdings(pool: &PgPool) {
    sqlx::query("DELETE FROM exchange_holdings WHERE exchange_id = 'bitunix'")
        .execute(pool)
        .await
        .expect("clear holdings");
    sqlx::query("DELETE FROM portfolio_baselines WHERE exchange_id = 'bitunix'")
        .execute(pool)
        .await
        .expect("clear baseline");
}

#[tokio::test]
async fn bo_futures_wallet_priced_subtotal_nonzero() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0023 BO integration test");
            return;
        }
    };

    clear_bitunix_holdings(&pool).await;
    enable_bitunix(&pool).await;

    let repo = ExchangeRepository::new(pool.clone());
    repo.upsert_holdings(
        "bitunix",
        &[ExchangeHolding {
            asset: "USDT".into(),
            quantity: 2000.0,
            product_type: "futures".into(),
            market_value_usd: Some(2000.0),
            unrealized_pnl: None,
            avg_cost: None,
            payload: json!({ "marginCoin": "USDT", "accountEquity": "2000" }),
        }],
    )
    .await
    .expect("upsert futures wallet");

    repo.update_holding_eur("bitunix", "USDT", "futures", Some(1840.0), None, None, None)
        .await
        .expect("price futures wallet");

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );
    let breakdown = wealth
        .compute_extended(Some(&repo), None, None)
        .await
        .expect("wealth breakdown");

    assert!(
        breakdown.crypto.subtotal_eur > 0.0,
        "crypto subtotal must include priced futures wallet"
    );
    let bitunix = breakdown
        .crypto
        .exchanges
        .iter()
        .find(|e| e.id == "bitunix")
        .expect("bitunix exchange summary");
    assert!(
        bitunix.subtotal_eur > 0.0,
        "bitunix card subtotal must not be zero"
    );
    assert!(!breakdown.crypto_placeholder);
}

#[tokio::test]
async fn bp_linear_exposure_eur_value_without_subtotal_merge() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0023 BP integration test");
            return;
        }
    };

    clear_bitunix_holdings(&pool).await;
    enable_bitunix(&pool).await;
    seed_fx_rate(&pool).await;
    let run_id = seed_sync_run(&pool).await;

    let repo = ExchangeRepository::new(pool.clone());
    repo.upsert_holdings(
        "bitunix",
        &[
            ExchangeHolding {
                asset: "USDT".into(),
                quantity: 1000.0,
                product_type: "futures".into(),
                market_value_usd: Some(1000.0),
                unrealized_pnl: None,
                avg_cost: None,
                payload: json!({ "marginCoin": "USDT" }),
            },
            ExchangeHolding {
                asset: "INJUSDT".into(),
                quantity: 1.5,
                product_type: "linear".into(),
                market_value_usd: None,
                unrealized_pnl: Some(-3.25),
                avg_cost: None,
                payload: json!({
                    "symbol": "INJUSDT",
                    "qty": "1.5",
                    "entryValue": "500.0",
                    "unrealizedPNL": "-3.25"
                }),
            },
        ],
    )
    .await
    .expect("upsert holdings");

    repo.update_holding_eur("bitunix", "USDT", "futures", Some(920.0), None, None, None)
        .await
        .expect("price wallet");

    let fx = FxService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );
    let portfolio = PortfolioEngine::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );
    portfolio
        .recompute_pnl(
            run_id,
            &fx,
            std::sync::Arc::new(ExchangeRepository::new(pool.clone())),
        )
        .await
        .expect("recompute pnl");

    let repo = ExchangeRepository::new(pool.clone());

    let exposure: Option<f64> = sqlx::query_scalar(
        "SELECT exposure_eur::float8 FROM exchange_holdings WHERE exchange_id = 'bitunix' AND asset = 'INJUSDT' AND product_type = 'linear'",
    )
    .fetch_one(&pool)
    .await
    .expect("linear exposure_eur");

    let market_value: Option<f64> = sqlx::query_scalar(
        "SELECT market_value_eur::float8 FROM exchange_holdings WHERE exchange_id = 'bitunix' AND asset = 'INJUSDT' AND product_type = 'linear'",
    )
    .fetch_one(&pool)
    .await
    .expect("linear market_value_eur");

    assert!(exposure.is_some(), "linear exposure_eur must be populated");
    assert!(exposure.unwrap() > 0.0, "linear exposure_eur must be positive");
    assert!(
        market_value.is_none(),
        "linear market_value_eur must remain NULL per DEC-0064"
    );

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );
    let breakdown = wealth
        .compute_extended(Some(&repo), Some(&portfolio), None)
        .await
        .expect("wealth breakdown");

    let linear_row = breakdown
        .crypto
        .holdings_all
        .iter()
        .find(|h| h.asset == "INJUSDT" && h.product_type == "linear")
        .expect("linear holdings_all row");
    assert!(
        linear_row.value_eur.is_some(),
        "holdings_all value_eur must be non-null for linear"
    );

    assert!(
        (breakdown.crypto.subtotal_eur - 920.0).abs() < 0.01,
        "subtotal must be wallet-only (920), got {}",
        breakdown.crypto.subtotal_eur
    );
    let exposure_sum: f64 = breakdown
        .crypto
        .holdings_all
        .iter()
        .filter(|h| h.product_type == "linear")
        .filter_map(|h| h.value_eur)
        .sum();
    assert!(
        breakdown.crypto.subtotal_eur < 920.0 + exposure_sum,
        "subtotal must not merge linear exposure_eur"
    );
}

#[tokio::test]
async fn bq_priced_wallet_baseline_total_return_pct() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0023 BQ integration test");
            return;
        }
    };

    clear_bitunix_holdings(&pool).await;
    enable_bitunix(&pool).await;
    let run_id = seed_sync_run(&pool).await;

    let repo = ExchangeRepository::new(pool.clone());
    repo.upsert_holdings(
        "bitunix",
        &[ExchangeHolding {
            asset: "USDT".into(),
            quantity: 2000.0,
            product_type: "futures".into(),
            market_value_usd: Some(2000.0),
            unrealized_pnl: None,
            avg_cost: None,
            payload: json!({ "marginCoin": "USDT" }),
        }],
    )
    .await
    .expect("upsert futures wallet");

    repo.update_holding_eur("bitunix", "USDT", "futures", Some(1840.0), None, None, None)
        .await
        .expect("price futures wallet");

    let fx = FxService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );
    let portfolio = PortfolioEngine::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        PortfolioConfig::default(),
    );

    let result = portfolio
        .recompute_pnl(
            run_id,
            &fx,
            std::sync::Arc::new(ExchangeRepository::new(pool.clone())),
        )
        .await
        .expect("recompute pnl");

    assert!(
        result.crypto_value_eur > 0.0,
        "crypto_value_eur must be positive when futures wallet priced"
    );
    assert!(
        result.total_return_pct.is_some(),
        "total_return_pct must be non-null when baseline captured on first priced recompute"
    );

    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );
    let breakdown = wealth
        .compute_extended(Some(&repo), Some(&portfolio), None)
        .await
        .expect("wealth breakdown");

    assert!(
        breakdown.pnl.total_return_pct.is_some(),
        "wealth API pnl.total_return_pct must be non-null"
    );
}

#[tokio::test]
async fn regression_wealth_list_shape_unchanged() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for bug0023 regression test");
            return;
        }
    };

    enable_bitunix(&pool).await;
    let wealth = WealthService::new(
        flow_finance_ai::db::DbPool::from_pool(pool.clone()),
        WealthConfig::default(),
        "EUR".into(),
    );
    let repo = ExchangeRepository::new(pool.clone());
    let breakdown = wealth
        .compute_extended(Some(&repo), None, None)
        .await
        .expect("wealth breakdown");

    assert_eq!(breakdown.reporting_currency, "EUR");
    assert!(breakdown.crypto.holdings_all.len() <= 50);
    for row in &breakdown.crypto.holdings_all {
        assert!(!row.asset.is_empty());
        assert!(!row.product_type.is_empty());
    }
}
