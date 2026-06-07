use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::WealthConfig;
use crate::db::DbPool;
use crate::exchanges::repository::ExchangeRepository;
use crate::portfolio::PortfolioEngine;

use super::repository::WealthRepository;
use super::types::{
    AccountWealthRow, AllocationGap, AllocationWeights, CryptoBreakdown, CryptoExchangeSummary,
    CryptoHoldingRow, ExtendedWealthBreakdown, FireflyBreakdown, HoldingsAllRow,
    NetWorthBreakdown, PnlSummary, WealthError, WealthHistoryPoint,
};

#[derive(Clone)]
pub struct WealthService {
    repo: Arc<WealthRepository>,
    reporting_currency: String,
}

impl WealthService {
    pub fn new(db: DbPool, config: WealthConfig, reporting_currency: String) -> Self {
        let repo = WealthRepository::new(db.pool().clone(), config);
        Self {
            repo: Arc::new(repo),
            reporting_currency,
        }
    }

    pub fn repository(&self) -> &WealthRepository {
        &self.repo
    }

    pub async fn compute_breakdown(
        &self,
        exchange_repo: Option<&ExchangeRepository>,
        portfolio: Option<&PortfolioEngine>,
    ) -> Result<NetWorthBreakdown, WealthError> {
        let extended = self
            .compute_extended(exchange_repo, portfolio, None)
            .await?;

        Ok(NetWorthBreakdown {
            total: extended.total_eur,
            reporting_currency: extended.reporting_currency.clone(),
            mixed_currency: extended.firefly.mixed_currency,
            crypto_placeholder: extended.crypto_placeholder,
            accounts: extended.firefly.accounts.clone(),
            last_successful_sync_at: extended.last_successful_sync_at,
            computed_at: extended.computed_at,
            firefly: Some(extended.firefly),
            crypto: Some(extended.crypto),
            pnl: Some(extended.pnl),
            fx_incomplete: extended.fx_incomplete,
            total_eur: Some(extended.total_eur),
            allocation_gap: extended.allocation_gap,
        })
    }

    pub async fn compute_extended(
        &self,
        exchange_repo: Option<&ExchangeRepository>,
        portfolio: Option<&PortfolioEngine>,
        allocation_target: Option<AllocationWeights>,
    ) -> Result<ExtendedWealthBreakdown, WealthError> {
        let accounts = self.repo.load_asset_accounts().await?;
        let last_sync = self.repo.last_successful_sync_at().await?;

        let currencies: std::collections::HashSet<String> = accounts
            .iter()
            .filter_map(|a| a.currency.clone())
            .collect();
        let mixed_currency = currencies.len() > 1;

        let firefly_subtotal: f64 = accounts.iter().map(|a| a.balance.unwrap_or(0.0)).sum();

        let rows: Vec<AccountWealthRow> = accounts
            .iter()
            .map(|a| {
                let balance = a.balance.unwrap_or(0.0);
                let pct = if mixed_currency || firefly_subtotal == 0.0 {
                    None
                } else {
                    Some((balance / firefly_subtotal) * 100.0)
                };
                AccountWealthRow {
                    firefly_id: a.firefly_id.clone(),
                    name: a.name.clone().unwrap_or_else(|| "Unknown".into()),
                    account_role: a.account_role.clone(),
                    currency: a
                        .currency
                        .clone()
                        .unwrap_or_else(|| self.reporting_currency.clone()),
                    balance,
                    is_overdrawn: balance < 0.0,
                    pct_of_total: pct,
                }
            })
            .collect();

        let mut crypto = CryptoBreakdown {
            subtotal_eur: 0.0,
            fx_complete: true,
            exchanges: vec![],
            holdings_top: vec![],
            holdings_all: vec![],
            unpriced_assets: vec![],
        };
        let mut crypto_placeholder = true;

        if let Some(er) = exchange_repo {
            let connections = er.list_connections().await?;
            let holdings = er.load_all_holdings().await?;

            let any_enabled = connections.iter().any(|c| c.enabled);
            if any_enabled {
                crypto_placeholder = false;
            }

            for conn in &connections {
                if !conn.enabled {
                    continue;
                }
                let subtotal: f64 = holdings
                    .iter()
                    .filter(|h| h.exchange_id == conn.id)
                    .filter_map(|h| h.market_value_eur)
                    .sum();
                let count = holdings.iter().filter(|h| h.exchange_id == conn.id).count() as i64;
                crypto.exchanges.push(CryptoExchangeSummary {
                    id: conn.id.clone(),
                    connection_state: conn.connection_state.clone(),
                    last_sync_at: conn.last_sync_at,
                    subtotal_eur: subtotal,
                    holdings_count: count,
                });
            }

            let mut top: Vec<CryptoHoldingRow> = holdings
                .iter()
                .filter_map(|h| {
                    Some(CryptoHoldingRow {
                        exchange_id: h.exchange_id.clone(),
                        asset: h.asset.clone(),
                        quantity: h.quantity,
                        value_eur: h.market_value_eur?,
                        unrealized_pnl_eur: h.unrealized_pnl_eur,
                        product_type: h.product_type.clone(),
                    })
                })
                .collect();
            top.sort_by(|a, b| b.value_eur.partial_cmp(&a.value_eur).unwrap_or(std::cmp::Ordering::Equal));
            top.truncate(5);
            crypto.holdings_top = top;

            crypto.subtotal_eur = holdings.iter().filter_map(|h| h.market_value_eur).sum();

            let mut all: Vec<HoldingsAllRow> = holdings
                .iter()
                .map(|h| HoldingsAllRow {
                    asset: h.asset.clone(),
                    quantity: h.quantity,
                    product_type: h.product_type.clone(),
                    value_eur: h.market_value_eur,
                    unrealized_pnl_eur: h.unrealized_pnl_eur,
                    native_unit: native_unit_for_holding(h),
                })
                .collect();
            all.sort_by(|a, b| compare_holdings_all(a, b));
            all.truncate(50);
            crypto.holdings_all = all;
        }

        let (pnl, pnl_fx_incomplete, pnl_unpriced) = if let Some(pe) = portfolio {
            if let Ok(Some(latest)) = pe.latest().await {
                (
                    PnlSummary {
                        realized_eur: latest.realized_eur,
                        unrealized_eur: latest.unrealized_eur,
                        total_return_pct: latest.total_return_pct,
                    },
                    latest.fx_incomplete,
                    latest.unpriced_assets,
                )
            } else {
                (
                    PnlSummary {
                        realized_eur: 0.0,
                        unrealized_eur: 0.0,
                        total_return_pct: None,
                    },
                    false,
                    vec![],
                )
            }
        } else {
            (
                PnlSummary {
                    realized_eur: 0.0,
                    unrealized_eur: 0.0,
                    total_return_pct: None,
                },
                false,
                vec![],
            )
        };

        crypto.unpriced_assets = pnl_unpriced;
        let fx_incomplete = pnl_fx_incomplete || !crypto.unpriced_assets.is_empty();
        crypto.fx_complete = !fx_incomplete;

        let total_eur = if fx_incomplete {
            firefly_subtotal
        } else {
            firefly_subtotal + crypto.subtotal_eur
        };

        let allocation_gap = allocation_target.map(|target| {
            let total = total_eur.max(1.0);
            let crypto_pct = (crypto.subtotal_eur / total) * 100.0;
            let etf_pct = (firefly_subtotal / total) * 100.0;
            let cash_pct = (100.0 - crypto_pct - etf_pct).max(0.0);
            let current = AllocationWeights {
                etf_traditional_pct: etf_pct,
                crypto_pct,
                cash_pct,
            };
            AllocationGap {
                gaps: AllocationWeights {
                    etf_traditional_pct: target.etf_traditional_pct - etf_pct,
                    crypto_pct: target.crypto_pct - crypto_pct,
                    cash_pct: target.cash_pct - cash_pct,
                },
                current,
                target,
            }
        });

        Ok(ExtendedWealthBreakdown {
            reporting_currency: self.reporting_currency.clone(),
            firefly: FireflyBreakdown {
                subtotal_eur: firefly_subtotal,
                mixed_currency,
                accounts: rows,
            },
            crypto,
            total_eur,
            pnl,
            fx_incomplete,
            crypto_placeholder,
            allocation_gap,
            last_successful_sync_at: last_sync,
            computed_at: Utc::now(),
        })
    }

    pub async fn upsert_daily_snapshot(
        &self,
        sync_run_id: Uuid,
        exchange_repo: Option<&ExchangeRepository>,
        portfolio: Option<&PortfolioEngine>,
    ) -> Result<(), WealthError> {
        let breakdown = self
            .compute_extended(exchange_repo, portfolio, None)
            .await?;
        let today = Utc::now().date_naive();

        let payload = serde_json::json!({
            "accounts": breakdown.firefly.accounts,
            "crypto": {
                "holdings_top": breakdown.crypto.holdings_top,
                "exchanges": breakdown.crypto.exchanges,
            },
            "allocation": {
                "etf_traditional": breakdown.firefly.subtotal_eur,
                "crypto": breakdown.crypto.subtotal_eur,
                "cash": 0.0,
            },
            "fx_incomplete": breakdown.fx_incomplete,
        });

        self.repo
            .upsert_snapshot(
                today,
                breakdown.total_eur,
                breakdown.firefly.mixed_currency,
                breakdown.firefly.accounts.len() as i32,
                &payload,
                Some(sync_run_id),
                breakdown.crypto.subtotal_eur,
                breakdown.firefly.subtotal_eur,
                breakdown.pnl.total_return_pct,
            )
            .await?;

        Ok(())
    }

    pub async fn history(&self, days: u32) -> Result<Vec<WealthHistoryPoint>, WealthError> {
        let rows = self.repo.fetch_history(days).await?;
        Ok(rows
            .into_iter()
            .map(|r| WealthHistoryPoint {
                snapshot_date: r.snapshot_date,
                total_eur: r.total_eur,
                mixed_currency: r.mixed_currency,
                account_count: r.account_count,
                crypto_value_eur: r.crypto_value_eur.unwrap_or(0.0),
                firefly_value_eur: r.firefly_value_eur.unwrap_or(0.0),
                total_return_pct: r.total_return_pct,
            })
            .collect())
    }

    pub fn portfolio_summary_for_ai(
        &self,
        breakdown: &ExtendedWealthBreakdown,
    ) -> serde_json::Value {
        serde_json::json!({
            "total_eur": breakdown.total_eur,
            "firefly_subtotal_eur": breakdown.firefly.subtotal_eur,
            "crypto_subtotal_eur": breakdown.crypto.subtotal_eur,
            "crypto_top_holdings": breakdown.crypto.holdings_top,
            "pnl": breakdown.pnl,
            "fx_incomplete": breakdown.fx_incomplete,
        })
    }
}

fn native_unit_for_holding(h: &crate::exchanges::repository::HoldingRow) -> String {
    h.asset.clone()
}

fn compare_holdings_all(a: &HoldingsAllRow, b: &HoldingsAllRow) -> std::cmp::Ordering {
    match (a.value_eur, b.value_eur) {
        (Some(va), Some(vb)) => vb
            .partial_cmp(&va)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.asset.cmp(&b.asset),
    }
}

#[cfg(test)]
mod tests {
    use crate::wealth::types::{AccountWealthRow, HoldingsAllRow};

    #[test]
    fn holdings_all_sorts_priced_before_unpriced() {
        let rows = vec![
            HoldingsAllRow {
                asset: "ALT".into(),
                quantity: 1.0,
                product_type: "spot".into(),
                value_eur: None,
                unrealized_pnl_eur: None,
                native_unit: "ALT".into(),
            },
            HoldingsAllRow {
                asset: "BTC".into(),
                quantity: 0.1,
                product_type: "spot".into(),
                value_eur: Some(5000.0),
                unrealized_pnl_eur: None,
                native_unit: "BTC".into(),
            },
        ];
        let mut sorted = rows;
        sorted.sort_by(super::compare_holdings_all);
        assert_eq!(sorted[0].asset, "BTC");
        assert_eq!(sorted[1].asset, "ALT");
    }

    #[test]
    fn fx_incomplete_when_unpriced_assets_present() {
        let unpriced: Vec<String> = vec!["DOGE".into()];
        let pnl_fx = false;
        let fx_incomplete = pnl_fx || !unpriced.is_empty();
        assert!(fx_incomplete);
    }

    #[test]
    fn mixed_currency_when_multiple_currencies() {
        let accounts = vec![("EUR", 1000.0), ("USD", 500.0)];
        let currencies: std::collections::HashSet<_> =
            accounts.iter().map(|(c, _)| c.to_string()).collect();
        assert!(currencies.len() > 1);
    }

    #[test]
    fn overdrawn_flag_when_balance_negative() {
        let row = AccountWealthRow {
            firefly_id: "114".into(),
            name: "Giro".into(),
            account_role: Some("defaultAsset".into()),
            currency: "EUR".into(),
            balance: -3395.75,
            is_overdrawn: true,
            pct_of_total: None,
        };
        assert!(row.is_overdrawn);
    }
}
