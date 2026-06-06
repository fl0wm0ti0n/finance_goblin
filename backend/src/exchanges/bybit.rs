use std::time::Instant;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use serde_json::Value;

use crate::config::ExchangeInstanceConfig;

use super::http::{hmac_sha256_hex, ExchangeHttpClient};
use super::trait_def::ExchangeConnector;
use super::types::{
    ConnectionTest, ExchangeError, ExchangeFundingEvent, ExchangeHolding, ExchangeSyncState,
    ExchangeTrade, ExchangeTransfer,
};

pub struct BybitConnector {
    config: ExchangeInstanceConfig,
    http: ExchangeHttpClient,
}

impl BybitConnector {
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

    async fn signed_get(&self, path: &str, query: &str) -> Result<Value, ExchangeError> {
        let (api_key, secret) = self.creds()?;
        let ts = Utc::now().timestamp_millis();
        let recv = 5000;
        let q = if query.is_empty() {
            format!("accountType=UNIFIED")
        } else {
            query.to_string()
        };
        let sign_payload = format!("{ts}{api_key}{recv}{q}");
        let sig = hmac_sha256_hex(&secret, &sign_payload);
        let url = format!(
            "{}{path}?{q}",
            self.config.base_url.trim_end_matches('/')
        );
        let headers = vec![
            ("X-BAPI-API-KEY".to_string(), api_key),
            ("X-BAPI-SIGN".to_string(), sig),
            ("X-BAPI-TIMESTAMP".to_string(), ts.to_string()),
            ("X-BAPI-RECV-WINDOW".to_string(), recv.to_string()),
        ];
        let body = self.http.get_with_backoff(&url, headers).await?;
        if body["retCode"].as_i64().unwrap_or(0) != 0 {
            return Err(ExchangeError::Api(body["retMsg"].as_str().unwrap_or("error").into()));
        }
        Ok(body["result"].clone())
    }
}

#[async_trait]
impl ExchangeConnector for BybitConnector {
    fn exchange_id(&self) -> &'static str {
        "bybit"
    }

    async fn test_connection(&self) -> Result<ConnectionTest, ExchangeError> {
        let start = Instant::now();
        match self.signed_get("/v5/account/wallet-balance", "").await {
            Ok(_) => Ok(ConnectionTest {
                ok: true,
                latency_ms: start.elapsed().as_millis() as u64,
                message: "Unified wallet read OK".into(),
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
        let result = self.signed_get("/v5/account/wallet-balance", "").await?;
        let mut holdings = Vec::new();
        state.active_symbols.clear();

        if let Some(list) = result["list"].as_array() {
            for acct in list {
                if let Some(coins) = acct["coin"].as_array() {
                    for c in coins {
                        let asset = c["coin"].as_str().unwrap_or_default().to_string();
                        let qty: f64 = c["walletBalance"]
                            .as_str()
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0.0);
                        if qty <= 0.0 {
                            continue;
                        }
                        let usd_value: Option<f64> = c["usdValue"]
                            .as_str()
                            .and_then(|s| s.parse().ok());
                        if asset != "USDT" {
                            state.active_symbols.push(format!("{asset}USDT"));
                        }
                        holdings.push(ExchangeHolding {
                            asset,
                            quantity: qty,
                            product_type: "spot".into(),
                            market_value_usd: usd_value,
                            unrealized_pnl: None,
                            avg_cost: None,
                            payload: c.clone(),
                        });
                    }
                }
            }
        }
        Ok(holdings)
    }

    async fn sync_positions(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError> {
        let result = match self
            .signed_get("/v5/position/list", "category=linear&settleCoin=USDT")
            .await
        {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut holdings = Vec::new();
        if let Some(list) = result["list"].as_array() {
            for p in list {
                let qty: f64 = p["size"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                if qty <= 0.0 {
                    continue;
                }
                let symbol = p["symbol"].as_str().unwrap_or_default().to_string();
                state.active_symbols.push(symbol.clone());
                let unrealized: f64 = p["unrealisedPnl"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                holdings.push(ExchangeHolding {
                    asset: symbol,
                    quantity: qty,
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
            let query = format!("category=spot&symbol={symbol}&startTime={start_ms}&limit=100");
            let result = match self.signed_get("/v5/execution/list", &query).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(list) = result["list"].as_array() {
                for t in list {
                    let ts = t["execTime"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0i64);
                    let executed_at = Utc
                        .timestamp_millis_opt(ts)
                        .single()
                        .unwrap_or_else(Utc::now);
                    all.push(ExchangeTrade {
                        external_id: t["execId"].as_str().unwrap_or_default().into(),
                        symbol: symbol.clone(),
                        side: t["side"].as_str().unwrap_or("Buy").to_lowercase(),
                        quantity: t["execQty"]
                            .as_str()
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0.0),
                        price: t["execPrice"]
                            .as_str()
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0.0),
                        fee: t["execFee"]
                            .as_str()
                            .map(|s| s.parse().unwrap_or(0.0)),
                        fee_asset: t["feeCurrency"].as_str().map(String::from),
                        realized_pnl: None,
                        executed_at,
                        payload: t.clone(),
                    });
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
        let since = state
            .last_funding_time
            .unwrap_or_else(|| Utc::now() - chrono::Duration::days(90));
        let start_ms = since.timestamp_millis();
        let query = format!("category=linear&startTime={start_ms}&limit=100");
        let result = match self.signed_get("/v5/execution/list", &query).await {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut events = Vec::new();
        if let Some(list) = result["list"].as_array() {
            for e in list {
                if e["execType"].as_str() != Some("Funding") {
                    continue;
                }
                let ts = e["execTime"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0i64);
                let executed_at = Utc
                    .timestamp_millis_opt(ts)
                    .single()
                    .unwrap_or_else(Utc::now);
                events.push(ExchangeFundingEvent {
                    external_id: e["execId"].as_str().unwrap_or_default().into(),
                    symbol: e["symbol"].as_str().map(String::from),
                    amount: e["execFee"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0),
                    asset: "USDT".into(),
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
