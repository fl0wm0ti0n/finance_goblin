use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::PortfolioConfig;
use crate::db::DbPool;
use crate::exchanges::repository::ExchangeRepository;
use crate::fx::{ExchangePriceBook, FxService};

use super::baseline::BaselineService;
use super::pnl::{compute_hybrid_pnl, PnlBreakdown};
use super::repository::PortfolioRepository;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PnlResult {
    pub realized_eur: f64,
    pub unrealized_eur: f64,
    pub total_return_pct: Option<f64>,
    pub crypto_value_eur: f64,
    pub fx_incomplete: bool,
    pub unpriced_assets: Vec<String>,
}

#[derive(Clone)]
pub struct PortfolioEngine {
    repo: Arc<PortfolioRepository>,
    config: PortfolioConfig,
}

impl PortfolioEngine {
    pub fn new(db: DbPool, config: PortfolioConfig) -> Self {
        Self {
            repo: Arc::new(PortfolioRepository::new(db.pool().clone())),
            config,
        }
    }

    pub fn repository(&self) -> &PortfolioRepository {
        &self.repo
    }

    pub async fn run_trade_retention(&self, exchange_repo: &ExchangeRepository) -> Result<u64, sqlx::Error> {
        exchange_repo
            .prune_trades(self.config.trade_retention_days as i32)
            .await
    }

    pub async fn recompute_pnl(
        &self,
        run_id: Uuid,
        fx: &FxService,
        exchange_repo: Arc<ExchangeRepository>,
    ) -> Result<PnlResult, sqlx::Error> {
        let price_book = ExchangePriceBook::default();
        let breakdown = compute_hybrid_pnl(&exchange_repo, fx, &price_book).await?;

        let baseline_svc = BaselineService::new(exchange_repo.clone());
        let baseline = baseline_svc.total_baseline_eur().await?;
        let total_return_pct = if baseline > 0.0 {
            Some(((breakdown.crypto_value_eur - baseline) / baseline) * 100.0)
        } else {
            None
        };

        for exchange_id in ["binance", "bybit", "bitunix"] {
            let holdings = exchange_repo.load_all_holdings().await?;
            let exchange_total: f64 = holdings
                .iter()
                .filter(|h| h.exchange_id == exchange_id)
                .filter_map(|h| h.market_value_eur)
                .sum();
            if exchange_total > 0.0 {
                baseline_svc
                    .capture_if_missing(exchange_id, exchange_total, run_id)
                    .await?;
            }
        }

        let today = Utc::now().date_naive();
        let payload = serde_json::json!({
            "unpriced_assets": breakdown.unpriced_assets,
            "fx_incomplete": breakdown.fx_incomplete,
        });
        self.repo
            .upsert_pnl_snapshot(
                today,
                run_id,
                breakdown.realized_eur,
                breakdown.unrealized_eur,
                total_return_pct,
                breakdown.crypto_value_eur,
                &payload,
            )
            .await?;

        Ok(PnlResult {
            realized_eur: breakdown.realized_eur,
            unrealized_eur: breakdown.unrealized_eur,
            total_return_pct,
            crypto_value_eur: breakdown.crypto_value_eur,
            fx_incomplete: breakdown.fx_incomplete,
            unpriced_assets: breakdown.unpriced_assets.clone(),
        })
    }

    pub async fn latest(&self) -> Result<Option<PnlResult>, sqlx::Error> {
        Ok(self.repo.latest_pnl().await?.map(|r| {
            let unpriced_assets: Vec<String> = r
                .payload
                .get("unpriced_assets")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let fx_incomplete = r
                .payload
                .get("fx_incomplete")
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| !unpriced_assets.is_empty());
            PnlResult {
                realized_eur: r.realized_pnl_eur,
                unrealized_eur: r.unrealized_pnl_eur,
                total_return_pct: r.total_return_pct,
                crypto_value_eur: r.crypto_value_eur,
                fx_incomplete,
                unpriced_assets,
            }
        }))
    }
}
