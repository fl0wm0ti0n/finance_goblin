use std::sync::Arc;

use tracing::{info, warn};
use uuid::Uuid;

use crate::config::ExchangesConfig;
use crate::db::DbPool;
use crate::fx::FxService;
use crate::portfolio::PortfolioEngine;

use super::binance::BinanceConnector;
use super::bitunix::BitunixConnector;
use super::bybit::BybitConnector;
use super::repository::ExchangeRepository;
use super::trait_def::ExchangeConnector;
use super::types::{
    ConnectionTest, ExchangeError, ExchangeListItem, ExchangeSyncState,
};

#[derive(Clone)]
pub struct ExchangeService {
    repo: Arc<ExchangeRepository>,
    config: ExchangesConfig,
    connectors: Vec<Arc<dyn ExchangeConnector>>,
    fx: FxService,
    portfolio: PortfolioEngine,
    last_tests: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ConnectionTest>>>,
}

impl ExchangeService {
    pub fn new(
        db: DbPool,
        config: ExchangesConfig,
        fx: FxService,
        portfolio: PortfolioEngine,
    ) -> Self {
        let repo = Arc::new(ExchangeRepository::new(db.pool().clone()));
        let connectors = Self::build_connectors(&config);

        Self {
            repo,
            config,
            connectors,
            fx,
            portfolio,
            last_tests: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn build_connectors(config: &ExchangesConfig) -> Vec<Arc<dyn ExchangeConnector>> {
        let mut connectors: Vec<Arc<dyn ExchangeConnector>> = Vec::new();
        if config.binance.effective_enabled() {
            connectors.push(Arc::new(BinanceConnector::new(config.binance.clone())));
        }
        if config.bybit.effective_enabled() {
            connectors.push(Arc::new(BybitConnector::new(config.bybit.clone())));
        }
        if config.bitunix.effective_enabled() {
            connectors.push(Arc::new(BitunixConnector::new(config.bitunix.clone())));
        }
        connectors
    }

    pub fn repository(&self) -> Arc<ExchangeRepository> {
        self.repo.clone()
    }

    pub async fn mirror_enabled_at_startup(&self) -> Result<(), sqlx::Error> {
        for id in ["binance", "bybit", "bitunix"] {
            let enabled = match id {
                "binance" => self.config.binance.effective_enabled(),
                "bybit" => self.config.bybit.effective_enabled(),
                "bitunix" => self.config.bitunix.effective_enabled(),
                _ => false,
            };
            self.repo.set_enabled(id, enabled).await?;
            let configured = match id {
                "binance" => self.config.binance.configured(),
                "bybit" => self.config.bybit.configured(),
                "bitunix" => self.config.bitunix.configured(),
                _ => false,
            };
            let state = if !enabled {
                "disabled"
            } else if configured {
                "not_configured"
            } else {
                "not_configured"
            };
            if enabled && configured {
                self.repo
                    .update_connection_state(id, "idle", None)
                    .await?;
            } else {
                self.repo.update_connection_state(id, state, None).await?;
            }
        }
        Ok(())
    }

    pub async fn list_connections(&self) -> Result<Vec<ExchangeListItem>, sqlx::Error> {
        let rows = self.repo.list_connections().await?;
        let mut out = Vec::new();
        for row in rows {
            let counts = self.repo.entity_counts(&row.id).await?;
            out.push(ExchangeListItem {
                id: row.id,
                enabled: row.enabled,
                connection_state: row.connection_state,
                last_sync_at: row.last_sync_at,
                last_error: row.last_error,
                counts,
            });
        }
        Ok(out)
    }

    pub async fn test_connection(&self, exchange_id: &str) -> Result<ConnectionTest, ExchangeError> {
        let connector = self
            .connectors
            .iter()
            .find(|c| c.exchange_id() == exchange_id)
            .ok_or_else(|| ExchangeError::Api(format!("unknown exchange: {exchange_id}")))?;

        let result = connector.test_connection().await?;
        self.last_tests
            .write()
            .await
            .insert(exchange_id.to_string(), result.clone());
        Ok(result)
    }

    pub async fn last_test(&self, exchange_id: &str) -> Option<ConnectionTest> {
        self.last_tests.read().await.get(exchange_id).cloned()
    }

    pub async fn run_post_sync(&self, run_id: Uuid) -> Result<bool, ExchangeError> {
        if !self.config.enabled {
            return Ok(false);
        }

        let mut fx_incomplete = false;

        for connector in &self.connectors {
            let exchange_id = connector.exchange_id();
            info!(%exchange_id, %run_id, "exchange sync started");

            let mut state = self.repo.get_sync_state(exchange_id).await?;

            let result = async {
                let balances = connector.sync_balances(&mut state).await?;
                self.repo.upsert_holdings(exchange_id, &balances).await?;

                let positions = connector.sync_positions(&mut state).await?;
                self.repo.upsert_holdings(exchange_id, &positions).await?;

                let trades = connector.sync_trades(&mut state).await?;
                self.repo.upsert_trades(exchange_id, &trades).await?;

                let transfers = connector.sync_transfers(&mut state).await?;
                self.repo.upsert_transfers(exchange_id, &transfers).await?;

                let funding = connector.sync_funding(&mut state).await?;
                self.repo.upsert_funding(exchange_id, &funding).await?;

                self.repo.update_sync_state(exchange_id, &state).await?;
                Ok::<(), ExchangeError>(())
            }
            .await;

            match result {
                Ok(()) => {
                    self.repo
                        .update_connection_state(exchange_id, "connected", None)
                        .await?;
                    info!(%exchange_id, %run_id, "exchange sync succeeded");
                }
                Err(e) => {
                    warn!(?e, %exchange_id, %run_id, "exchange sync failed");
                    self.repo
                        .update_connection_state(exchange_id, "error", Some(&e.to_string()))
                        .await?;
                    fx_incomplete = true;
                }
            }
        }

        if let Err(e) = self
            .portfolio
            .recompute_pnl(run_id, &self.fx, self.repo.clone())
            .await
        {
            warn!(?e, %run_id, "portfolio PnL recompute failed");
            fx_incomplete = true;
        }

        Ok(fx_incomplete)
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled && !self.connectors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BitunixConfig, ExchangeInstanceConfig, ExchangesConfig};

    #[test]
    fn build_connectors_registers_bitunix_when_configured_and_toml_disabled() {
        let prev_key = std::env::var("BITUNIX_API_KEY").ok();
        let prev_secret = std::env::var("BITUNIX_API_SECRET").ok();
        std::env::set_var("BITUNIX_API_KEY", "k");
        std::env::set_var("BITUNIX_API_SECRET", "s");
        let config = ExchangesConfig {
            enabled: true,
            interval_seconds: 3600,
            binance: ExchangeInstanceConfig {
                enabled: false,
                api_key_env: "BINANCE_API_KEY".into(),
                api_secret_env: "BINANCE_API_SECRET".into(),
                base_url: "https://api.binance.com".into(),
            },
            bybit: ExchangeInstanceConfig {
                enabled: false,
                api_key_env: "BYBIT_API_KEY".into(),
                api_secret_env: "BYBIT_API_SECRET".into(),
                base_url: "https://api.bybit.com".into(),
            },
            bitunix: BitunixConfig {
                enabled: false,
                api_key_env: "BITUNIX_API_KEY".into(),
                api_secret_env: "BITUNIX_API_SECRET".into(),
                spot_base_url: "https://openapi.bitunix.com".into(),
                futures_base_url: "https://fapi.bitunix.com".into(),
                enabled_futures: false,
            },
        };
        let connectors = ExchangeService::build_connectors(&config);
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].exchange_id(), "bitunix");
        restore_env("BITUNIX_API_KEY", prev_key);
        restore_env("BITUNIX_API_SECRET", prev_secret);
    }

    fn restore_env(key: &str, prev: Option<String>) {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}
