use tracing::warn;

use crate::exchanges::repository::{ExchangeRepository, HoldingRow};
use crate::fx::{ExchangePriceBook, EurAmount, FxError, FxService};

use super::avg_cost::average_cost_from_trades;

#[derive(Debug, Clone)]
pub struct PnlBreakdown {
    pub realized_eur: f64,
    pub unrealized_eur: f64,
    pub crypto_value_eur: f64,
    pub fx_incomplete: bool,
    pub unpriced_assets: Vec<String>,
}

pub async fn compute_hybrid_pnl(
    repo: &ExchangeRepository,
    fx: &FxService,
    price_book: &ExchangePriceBook,
) -> Result<PnlBreakdown, sqlx::Error> {
    let holdings = repo.load_all_holdings().await?;
    let mut realized = 0.0f64;
    let mut unrealized = 0.0f64;
    let mut crypto_value = 0.0f64;
    let mut fx_incomplete = false;
    let mut unpriced = Vec::new();

    for h in &holdings {
        if h.product_type == "linear" {
            let upnl_usdt = parse_unrealized_pnl_usdt(&h.payload);
            let upnl_eur = match upnl_usdt {
                Some(pnl) => match fx.to_eur(pnl, "USDT", price_book).await {
                    Ok(v) => Some(v.eur),
                    Err(_) => {
                        fx_incomplete = true;
                        None
                    }
                },
                None => None,
            };
            let exposure_eur = match parse_entry_value_usdt(&h.payload) {
                Some(entry) => match fx.to_eur(entry, "USDT", price_book).await {
                    Ok(v) => Some(v.eur),
                    Err(_) => {
                        fx_incomplete = true;
                        None
                    }
                },
                None => None,
            };
            repo.update_holding_eur(
                &h.exchange_id,
                &h.asset,
                &h.product_type,
                None,
                upnl_eur,
                h.avg_cost_eur,
                exposure_eur,
            )
            .await?;
            if let Some(upnl) = upnl_eur {
                unrealized += upnl;
            }
            continue;
        }

        let value_eur = match holding_value_eur(h, fx, price_book).await {
            Ok(v) => {
                repo.update_holding_eur(
                    &h.exchange_id,
                    &h.asset,
                    &h.product_type,
                    Some(v.eur),
                    if h.product_type == "futures" {
                        None
                    } else {
                        h.unrealized_pnl_eur
                    },
                    h.avg_cost_eur,
                    None,
                )
                .await?;
                v.eur
            }
            Err(FxError::Unpriced(asset)) => {
                fx_incomplete = true;
                unpriced.push(asset);
                continue;
            }
            Err(_) => {
                fx_incomplete = true;
                continue;
            }
        };

        crypto_value += value_eur;

        if h.product_type == "spot" {
            if let Some(upnl) = h.unrealized_pnl_eur {
                unrealized += upnl;
            } else if let Some(avg) = h.avg_cost_eur {
                unrealized += value_eur - avg * h.quantity;
            }
        }

        let since = chrono::Utc::now() - chrono::Duration::days(90);
        let trades = repo.load_trades_since(&h.exchange_id, since).await?;
        let exchange_trades: Vec<_> = trades
            .iter()
            .filter(|t| t.symbol.contains(&h.asset))
            .map(|t| crate::exchanges::types::ExchangeTrade {
                external_id: String::new(),
                symbol: t.symbol.clone(),
                side: t.side.clone(),
                quantity: t.quantity,
                price: t.price,
                fee: t.fee,
                fee_asset: t.fee_asset.clone(),
                realized_pnl: t.realized_pnl,
                executed_at: t.executed_at,
                payload: serde_json::json!({}),
            })
            .collect();

        let local_realized: f64 = exchange_trades
            .iter()
            .filter_map(|t| t.realized_pnl)
            .sum();

        let exchange_reported: f64 = holdings
            .iter()
            .filter(|x| x.exchange_id == h.exchange_id)
            .filter_map(|x| x.unrealized_pnl_eur)
            .sum();

        if exchange_reported.abs() > 0.0 && local_realized.abs() > 0.0 {
            let drift = ((exchange_reported - local_realized) / exchange_reported.abs()).abs();
            if drift > 0.01 {
                warn!(
                    exchange_id = %h.exchange_id,
                    drift_pct = drift * 100.0,
                    "pnl_reconciliation_warning"
                );
                realized += exchange_reported;
            } else {
                realized += local_realized;
            }
        } else {
            realized += local_realized;
        }
    }

    Ok(PnlBreakdown {
        realized_eur: realized,
        unrealized_eur: unrealized,
        crypto_value_eur: crypto_value,
        fx_incomplete,
        unpriced_assets: unpriced,
    })
}

async fn holding_value_eur(
    h: &HoldingRow,
    fx: &FxService,
    price_book: &ExchangePriceBook,
) -> Result<EurAmount, FxError> {
    if let Some(mv) = h.market_value_eur {
        return Ok(EurAmount { eur: mv });
    }

    fx.to_eur(h.quantity, &h.asset, price_book).await
}

pub fn parse_entry_value_usdt(payload: &serde_json::Value) -> Option<f64> {
    const KEYS: &[&str] = &["entryValue", "entry_value"];
    for key in KEYS {
        if let Some(v) = payload.get(*key) {
            if let Some(n) = v.as_f64() {
                return Some(n);
            }
            if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse() {
                    return Some(n);
                }
            }
        }
    }
    None
}

pub fn parse_unrealized_pnl_usdt(payload: &serde_json::Value) -> Option<f64> {
    const KEYS: &[&str] = &[
        "unrealizedPNL",
        "unrealizedPnl",
        "unrealizedProfit",
        "crossUnPnl",
        "unPnl",
        "profit",
    ];
    for key in KEYS {
        if let Some(v) = payload.get(*key) {
            if let Some(n) = v.as_f64() {
                return Some(n);
            }
            if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse() {
                    return Some(n);
                }
            }
        }
    }
    None
}

pub fn compute_avg_cost_fallback(
    trades: &[crate::exchanges::types::ExchangeTrade],
    asset: &str,
    qty: f64,
    mark_eur: f64,
) -> f64 {
    if let Some(avg) = average_cost_from_trades(trades, asset) {
        mark_eur - avg * qty
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avg_cost_fallback_unrealized() {
        use chrono::Utc;
        let trades = vec![crate::exchanges::types::ExchangeTrade {
            external_id: "1".into(),
            symbol: "ETHUSDT".into(),
            side: "buy".into(),
            quantity: 2.0,
            price: 1000.0,
            fee: None,
            fee_asset: None,
            realized_pnl: None,
            executed_at: Utc::now(),
            payload: serde_json::json!({}),
        }];
        let upnl = compute_avg_cost_fallback(&trades, "ETH", 2.0, 2500.0);
        assert!((upnl - 500.0).abs() < 0.01);
    }

    #[test]
    fn parse_entry_value_usdt_bitunix_shape() {
        let payload = serde_json::json!({ "entryValue": "1250.75" });
        assert_eq!(parse_entry_value_usdt(&payload), Some(1250.75));
    }

    #[test]
    fn parse_unrealized_pnl_usdt_bitunix_casing() {
        let payload = serde_json::json!({ "unrealizedPNL": "42.5" });
        assert_eq!(parse_unrealized_pnl_usdt(&payload), Some(42.5));
    }

    #[test]
    fn parse_unrealized_pnl_usdt_camel_case() {
        let payload = serde_json::json!({ "unrealizedPnl": 10.2 });
        assert_eq!(parse_unrealized_pnl_usdt(&payload), Some(10.2));
    }
}
