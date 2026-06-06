use std::time::Instant;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::config::ExchangeInstanceConfig;

use super::http::{hmac_sha256_hex, ExchangeHttpClient};
use super::trait_def::ExchangeConnector;
use super::types::{
    ConnectionTest, ExchangeError, ExchangeFundingEvent, ExchangeHolding, ExchangeSyncState,
    ExchangeTrade, ExchangeTransfer,
};

pub struct BinanceConnector {
    config: ExchangeInstanceConfig,
    http: ExchangeHttpClient,
}

impl BinanceConnector {
    pub fn new(config: ExchangeInstanceConfig) -> Self {
        Self {
            config,
            http: ExchangeHttpClient::new(),
        }
    }

    fn creds(&self) -> Result<(String, String), ExchangeError> {
        self.config
            .credentials()
            .ok_or(ExchangeError::NotConfigured)
    }

    async fn signed_get(&self, base: &str, path: &str, query: &str) -> Result<Value, ExchangeError> {
        let (api_key, secret) = self.creds()?;
        let ts = Utc::now().timestamp_millis();
        let q = if query.is_empty() {
            format!("timestamp={ts}")
        } else {
            format!("{query}&timestamp={ts}")
        };
        let sig = hmac_sha256_hex(&secret, &q);
        let url = format!("{base}{path}?{q}&signature={sig}");
        let headers = vec![("X-MBX-APIKEY".to_string(), api_key)];
        self.http.get_with_backoff(&url, headers).await
    }
}

#[async_trait]
impl ExchangeConnector for BinanceConnector {
    fn exchange_id(&self) -> &'static str {
        "binance"
    }

    async fn test_connection(&self) -> Result<ConnectionTest, ExchangeError> {
        let start = Instant::now();
        match self
            .signed_get(&self.config.base_url, "/api/v3/account", "")
            .await
        {
            Ok(_) => Ok(ConnectionTest {
                ok: true,
                latency_ms: start.elapsed().as_millis() as u64,
                message: "Balance read OK".into(),
                withdraw_enabled: None,
            }),
            Err(e) => Ok(ConnectionTest {
                ok: false,
                latency_ms: start.elapsed().as_millis() as u64,
                message: e.to_string(),
                withdraw_enabled: None,
            }),
        }
    }

    async fn sync_balances(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError> {
        let body = self
            .signed_get(&self.config.base_url, "/api/v3/account", "")
            .await?;
        let mut holdings = Vec::new();
        let mut symbols = Vec::new();

        if let Some(balances) = body["balances"].as_array() {
            for b in balances {
                let asset = b["asset"].as_str().unwrap_or_default().to_string();
                let free: f64 = b["free"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let locked: f64 = b["locked"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let qty = free + locked;
                if qty <= 0.0 {
                    continue;
                }
                if asset != "USDT" && asset != "USDC" && asset != "EUR" {
                    symbols.push(format!("{asset}USDT"));
                }
                holdings.push(ExchangeHolding {
                    asset: asset.clone(),
                    quantity: qty,
                    product_type: "spot".into(),
                    market_value_usd: None,
                    unrealized_pnl: None,
                    avg_cost: None,
                    payload: b.clone(),
                });
            }
        }

        state.active_symbols = symbols;
        Ok(holdings)
    }

    async fn sync_positions(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError> {
        let fapi = self.config.base_url.replace("api.binance.com", "fapi.binance.com");
        let body = match self.signed_get(&fapi, "/fapi/v2/account", "").await {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut holdings = Vec::new();
        if let Some(positions) = body["positions"].as_array() {
            for p in positions {
                let qty: f64 = p["positionAmt"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                if qty.abs() <= 0.0 {
                    continue;
                }
                let symbol = p["symbol"].as_str().unwrap_or_default().to_string();
                state.active_symbols.push(symbol.clone());
                let unrealized: f64 = p["unrealizedProfit"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                holdings.push(ExchangeHolding {
                    asset: symbol,
                    quantity: qty.abs(),
                    product_type: "linear".into(),
                    market_value_usd: None,
                    unrealized_pnl: Some(unrealized),
                    avg_cost: None,
                    payload: p.clone(),
                });
            }
        }
        Ok(holdings)
    }

    async fn sync_trades(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeTrade>, ExchangeError> {
        let since = state
            .last_trade_time
            .unwrap_or_else(|| Utc::now() - chrono::Duration::days(90));
        let start_ms = since.timestamp_millis();
        let mut all = Vec::new();

        for symbol in state.active_symbols.clone() {
            let query = format!("symbol={symbol}&startTime={start_ms}&limit=500");
            let body = match self
                .signed_get(&self.config.base_url, "/api/v3/myTrades", &query)
                .await
            {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(arr) = body.as_array() {
                for t in arr {
                    let trade = parse_binance_trade(t);
                    all.push(trade);
                }
            }
        }

        if let Some(latest) = all.iter().map(|t| t.executed_at).max() {
            state.last_trade_time = Some(latest);
        }
        Ok(all)
    }

    async fn sync_transfers(
        &self,
        _state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeTransfer>, ExchangeError> {
        Ok(vec![])
    }

    async fn sync_funding(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeFundingEvent>, ExchangeError> {
        let fapi = self.config.base_url.replace("api.binance.com", "fapi.binance.com");
        let since = state
            .last_funding_time
            .unwrap_or_else(|| Utc::now() - chrono::Duration::days(90));
        let start_ms = since.timestamp_millis();
        let query = format!("incomeType=FUNDING_FEE&startTime={start_ms}&limit=100");
        let body = match self.signed_get(&fapi, "/fapi/v1/income", &query).await {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut events = Vec::new();
        if let Some(arr) = body.as_array() {
            for e in arr {
                let ts = e["time"].as_i64().unwrap_or(0);
                let executed_at = Utc
                    .timestamp_millis_opt(ts)
                    .single()
                    .unwrap_or_else(Utc::now);
                events.push(ExchangeFundingEvent {
                    external_id: e["tranId"].to_string(),
                    symbol: e["symbol"].as_str().map(String::from),
                    amount: e["income"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0),
                    asset: e["asset"].as_str().unwrap_or("USDT").into(),
                    event_type: "funding_fee".into(),
                    executed_at,
                    payload: e.clone(),
                });
            }
        }

        if let Some(latest) = events.iter().map(|e| e.executed_at).max() {
            state.last_funding_time = Some(latest);
        }
        Ok(events)
    }
}

fn parse_binance_trade(t: &Value) -> ExchangeTrade {
    let ts = t["time"].as_i64().unwrap_or(0);
    let executed_at = Utc
        .timestamp_millis_opt(ts)
        .single()
        .unwrap_or_else(Utc::now);
    ExchangeTrade {
        external_id: t["id"].to_string(),
        symbol: t["symbol"].as_str().unwrap_or_default().into(),
        side: if t["isBuyer"].as_bool().unwrap_or(true) {
            "buy".into()
        } else {
            "sell".into()
        },
        quantity: t["qty"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0),
        price: t["price"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0),
        fee: t["commission"]
            .as_str()
            .map(|s| s.parse().unwrap_or(0.0)),
        fee_asset: t["commissionAsset"].as_str().map(String::from),
        realized_pnl: None,
        executed_at,
        payload: t.clone(),
    }
}
