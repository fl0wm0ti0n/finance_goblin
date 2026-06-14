use std::time::Instant;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;

use crate::config::BitunixConfig;

use super::http::{bitunix_futures_sign, bitunix_sign, ExchangeHttpClient};
use super::trait_def::ExchangeConnector;
use super::types::{
    ConnectionTest, ExchangeError, ExchangeFundingEvent, ExchangeHolding, ExchangeSyncState,
    ExchangeTrade, ExchangeTransfer,
};

pub struct BitunixConnector {
    config: BitunixConfig,
    http: ExchangeHttpClient,
}

impl BitunixConnector {
    pub fn new(config: BitunixConfig) -> Self {
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
        let q = if query.is_empty() {
            format!("timestamp={ts}")
        } else {
            format!("{query}&timestamp={ts}")
        };
        let sig = bitunix_sign(&secret, &q);
        let url = format!(
            "{}{path}?{q}&sign={sig}",
            self.config.spot_base_url.trim_end_matches('/')
        );
        let headers = vec![("api-key".to_string(), api_key)];
        self.http.get_with_backoff(&url, headers).await
    }

    async fn futures_signed_get(&self, path: &str, query: &str) -> Result<Value, ExchangeError> {
        let (api_key, secret) = self.creds()?;
        let nonce = futures_nonce();
        let ts = Utc::now().timestamp_millis().to_string();
        let query_params = sort_query_params(query);
        let sign = bitunix_futures_sign(&secret, &nonce, &ts, &api_key, &query_params, "");
        let base = self.config.futures_base_url.trim_end_matches('/');
        let url = if query.is_empty() {
            format!("{base}{path}")
        } else {
            format!("{base}{path}?{query}")
        };
        let headers = vec![
            ("api-key".to_string(), api_key),
            ("nonce".to_string(), nonce),
            ("timestamp".to_string(), ts),
            ("sign".to_string(), sign),
        ];
        self.http.get_with_backoff(&url, headers).await
    }

    fn parse_spot_balances(&self, body: &Value, state: &mut ExchangeSyncState) -> Vec<ExchangeHolding> {
        let mut holdings = Vec::new();
        state.active_symbols.clear();

        let balances = body["data"]["balances"]
            .as_array()
            .or_else(|| body["data"].as_array())
            .cloned()
            .unwrap_or_default();

        for b in balances {
            let asset = b["coin"]
                .as_str()
                .or_else(|| b["asset"].as_str())
                .unwrap_or_default()
                .to_string();
            let qty: f64 = b["available"]
                .as_str()
                .or_else(|| b["balance"].as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            if qty <= 0.0 {
                continue;
            }
            if asset != "USDT" {
                state.active_symbols.push(format!("{asset}USDT"));
            }
            holdings.push(ExchangeHolding {
                asset,
                quantity: qty,
                product_type: "spot".into(),
                market_value_usd: None,
                unrealized_pnl: None,
                avg_cost: None,
                payload: b.clone(),
            });
        }
        holdings
    }

    fn parse_futures_wallet(&self, body: &Value) -> Option<ExchangeHolding> {
        let data = body.get("data").unwrap_or(body);
        let account = resolve_futures_account(data);

        let equity = parse_f64_field(
            account,
            &["accountEquity", "totalEquity", "equity", "balance"],
        )
        .or_else(|| {
            let available = parse_f64_field(account, &["available", "avail"]).unwrap_or(0.0);
            let frozen = parse_f64_field(account, &["frozen", "freeze"]).unwrap_or(0.0);
            let margin = parse_f64_field(account, &["margin", "marginBalance"]).unwrap_or(0.0);
            let cross_upnl =
                parse_f64_field(account, &["crossUnrealizedPNL", "crossUnPnl"]).unwrap_or(0.0);
            let isolation_upnl = parse_f64_field(
                account,
                &["isolationUnrealizedPNL", "isolationUnPnl"],
            )
            .unwrap_or(0.0);
            let sum = available + frozen + margin + cross_upnl + isolation_upnl;
            if sum > 0.0 {
                Some(sum)
            } else {
                None
            }
        });

        let qty = equity?;
        if qty <= 0.0 {
            return None;
        }

        let asset = account["marginCoin"]
            .as_str()
            .or_else(|| account["coin"].as_str())
            .or_else(|| data["marginCoin"].as_str())
            .unwrap_or("USDT")
            .to_string();

        let unrealized = parse_f64_field(
            account,
            &[
                "unrealizedPNL",
                "unrealizedPnl",
                "crossUnrealizedPNL",
                "isolationUnrealizedPNL",
                "crossUnPnl",
                "unrealizedProfit",
                "unPnl",
            ],
        );

        let market_value_usd = if asset == "USDT" || asset == "USDC" {
            Some(qty)
        } else {
            None
        };

        Some(ExchangeHolding {
            asset,
            quantity: qty,
            product_type: "futures".into(),
            market_value_usd,
            unrealized_pnl: unrealized,
            avg_cost: None,
            payload: account.clone(),
        })
    }

    fn parse_futures_positions(
        &self,
        body: &Value,
        state: &mut ExchangeSyncState,
    ) -> Vec<ExchangeHolding> {
        let mut holdings = Vec::new();
        let positions = body["data"]["list"]
            .as_array()
            .or_else(|| body["data"]["positions"].as_array())
            .or_else(|| body["data"].as_array())
            .cloned()
            .unwrap_or_default();

        for p in positions {
            let qty: f64 = p["qty"]
                .as_str()
                .or_else(|| p["quantity"].as_str())
                .or_else(|| p["positionAmt"].as_str())
                .or_else(|| p["size"].as_str())
                .or_else(|| p["amount"].as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            if qty.abs() <= 0.0 {
                continue;
            }

            let symbol = p["symbol"]
                .as_str()
                .or_else(|| p["pair"].as_str())
                .or_else(|| p["instId"].as_str())
                .unwrap_or_default()
                .to_string();
            if !symbol.is_empty() {
                state.active_symbols.push(symbol.clone());
            }

            let unrealized = parse_f64_field(
                &p,
                &[
                    "unrealizedPNL",
                    "unrealizedPnl",
                    "unrealizedProfit",
                    "crossUnPnl",
                    "unPnl",
                    "profit",
                ],
            );

            holdings.push(ExchangeHolding {
                asset: symbol,
                quantity: qty.abs(),
                product_type: "linear".into(),
                market_value_usd: None,
                unrealized_pnl: unrealized,
                avg_cost: None,
                payload: p.clone(),
            });
        }
        holdings
    }
}

fn resolve_futures_account(data: &Value) -> &Value {
    if let Some(arr) = data.as_array() {
        for item in arr {
            if item.get("marginCoin").is_some() || item.get("available").is_some() {
                return item;
            }
        }
        if let Some(first) = arr.first() {
            return first;
        }
    }
    data.get("account").unwrap_or(data)
}

fn futures_nonce() -> String {
    Uuid::new_v4().simple().to_string()
}

fn sort_query_params(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let mut pairs: Vec<&str> = query.split('&').filter(|s| !s.is_empty()).collect();
    pairs.sort_unstable();
    pairs.join("&")
}

fn bitunix_response_ok(body: &Value) -> bool {
    match body.get("code") {
        Some(code) => code.as_i64() == Some(0) || code.as_str() == Some("0"),
        None => true,
    }
}

fn warn_futures_wallet_parse_skip(body: &Value) {
    let data = body.get("data").unwrap_or(body);
    let account = resolve_futures_account(data);
    let margin_coin = account
        .get("marginCoin")
        .or_else(|| data.get("marginCoin"))
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");
    let direct_keys = ["accountEquity", "totalEquity", "equity", "balance"];
    let available = parse_f64_field(account, &["available", "avail"]).unwrap_or(0.0);
    let frozen = parse_f64_field(account, &["frozen", "freeze"]).unwrap_or(0.0);
    let margin = parse_f64_field(account, &["margin", "marginBalance"]).unwrap_or(0.0);
    let cross_upnl =
        parse_f64_field(account, &["crossUnrealizedPNL", "crossUnPnl"]).unwrap_or(0.0);
    let isolation_upnl =
        parse_f64_field(account, &["isolationUnrealizedPNL", "isolationUnPnl"]).unwrap_or(0.0);
    warn!(
        margin_coin = margin_coin,
        equity_keys_tried = ?direct_keys,
        available = available,
        frozen = frozen,
        margin = margin,
        cross_unrealized_pnl = cross_upnl,
        isolation_unrealized_pnl = isolation_upnl,
        derived_sum = available + frozen + margin + cross_upnl + isolation_upnl,
        "bitunix futures wallet parse skipped"
    );
}

fn parse_f64_field(obj: &Value, keys: &[&str]) -> Option<f64> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
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

#[async_trait]
impl ExchangeConnector for BitunixConnector {
    fn exchange_id(&self) -> &'static str {
        "bitunix"
    }

    async fn test_connection(&self) -> Result<ConnectionTest, ExchangeError> {
        let start = Instant::now();
        let spot_result = self.signed_get("/api/spot/v1/user/account", "").await;

        if !self.config.effective_enabled_futures() {
            return match spot_result {
                Ok(_) => Ok(ConnectionTest {
                    ok: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: "Spot balance read OK".into(),
                    withdraw_enabled: None,
                }),
                Err(e) => Ok(ConnectionTest {
                    ok: false,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: format!("Spot: {e}"),
                    withdraw_enabled: None,
                }),
            };
        }

        match spot_result {
            Ok(_) => {
                let futures_result = self
                    .futures_signed_get("/api/v1/futures/account", "marginCoin=USDT")
                    .await;
                let latency_ms = start.elapsed().as_millis() as u64;
                match futures_result {
                    Ok(_) => Ok(ConnectionTest {
                        ok: true,
                        latency_ms,
                        message: "Spot: OK; Futures: OK".into(),
                        withdraw_enabled: None,
                    }),
                    Err(e) => Ok(ConnectionTest {
                        ok: true,
                        latency_ms,
                        message: format!("Spot: OK; Futures: {e}"),
                        withdraw_enabled: None,
                    }),
                }
            }
            Err(e) => Ok(ConnectionTest {
                ok: false,
                latency_ms: start.elapsed().as_millis() as u64,
                message: format!("Spot: {e}"),
                withdraw_enabled: None,
            }),
        }
    }

    async fn sync_balances(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError> {
        let body = self.signed_get("/api/spot/v1/user/account", "").await?;
        let mut holdings = self.parse_spot_balances(&body, state);

        if self.config.effective_enabled_futures() {
            match self
                .futures_signed_get("/api/v1/futures/account", "marginCoin=USDT")
                .await
            {
                Ok(futures_body) => {
                    if !bitunix_response_ok(&futures_body) {
                        warn!(
                            code = ?futures_body.get("code"),
                            msg = ?futures_body.get("msg"),
                            "bitunix futures wallet response rejected"
                        );
                    } else if let Some(h) = self.parse_futures_wallet(&futures_body) {
                        holdings.push(h);
                    } else {
                        warn_futures_wallet_parse_skip(&futures_body);
                    }
                }
                Err(e) => {
                    warn!(error = %e, "bitunix futures wallet sync failed");
                }
            }
        }

        Ok(holdings)
    }

    async fn sync_positions(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError> {
        if !self.config.effective_enabled_futures() {
            return Ok(vec![]);
        }

        let body = self
            .futures_signed_get("/api/v1/futures/position/get_pending_positions", "")
            .await?;
        Ok(self.parse_futures_positions(&body, state))
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
            let query = format!("symbol={symbol}&startTime={start_ms}&limit=100");
            let body = match self.signed_get("/api/spot/v1/order/trade_list", &query).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let trades = body["data"]["list"]
                .as_array()
                .or_else(|| body["data"].as_array())
                .cloned()
                .unwrap_or_default();

            for t in trades {
                let ts = t["time"].as_i64().unwrap_or(start_ms);
                let executed_at = Utc
                    .timestamp_millis_opt(ts)
                    .single()
                    .unwrap_or_else(Utc::now);
                all.push(ExchangeTrade {
                    external_id: t["id"].to_string(),
                    symbol: symbol.clone(),
                    side: t["side"].as_str().unwrap_or("buy").to_lowercase(),
                    quantity: t["qty"]
                        .as_str()
                        .or_else(|| t["quantity"].as_str())
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0),
                    price: t["price"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0),
                    fee: t["fee"].as_str().map(|s| s.parse().unwrap_or(0.0)),
                    fee_asset: t["feeCoin"].as_str().map(String::from),
                    realized_pnl: None,
                    executed_at,
                    payload: t.clone(),
                });
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
        _state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeFundingEvent>, ExchangeError> {
        if !self.config.effective_enabled_futures() {
            return Ok(vec![]);
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use wiremock::matchers::{header_exists, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct TestCreds {
        key_env: String,
        secret_env: String,
        prev_key: Option<String>,
        prev_secret: Option<String>,
        prev_futures: Option<String>,
    }

    impl TestCreds {
        fn new(suffix: &str) -> Self {
            let key_env = format!("BITUNIX_TEST_KEY_{suffix}");
            let secret_env = format!("BITUNIX_TEST_SECRET_{suffix}");
            let prev_key = std::env::var(&key_env).ok();
            let prev_secret = std::env::var(&secret_env).ok();
            let prev_futures = std::env::var("BITUNIX_ENABLED_FUTURES").ok();
            std::env::set_var(&key_env, "test-key");
            std::env::set_var(&secret_env, "test-secret");
            std::env::remove_var("BITUNIX_ENABLED_FUTURES");
            Self {
                key_env,
                secret_env,
                prev_key,
                prev_secret,
                prev_futures,
            }
        }

        fn config(&self, futures_base: &str, spot_base: &str, enabled_futures: bool) -> BitunixConfig {
            BitunixConfig {
                enabled: true,
                api_key_env: self.key_env.clone(),
                api_secret_env: self.secret_env.clone(),
                spot_base_url: spot_base.into(),
                futures_base_url: futures_base.into(),
                enabled_futures,
            }
        }

        fn disable_futures(&self) {
            std::env::set_var("BITUNIX_ENABLED_FUTURES", "false");
        }
    }

    impl Drop for TestCreds {
        fn drop(&mut self) {
            restore_env(&self.key_env, self.prev_key.take());
            restore_env(&self.secret_env, self.prev_secret.take());
            restore_env("BITUNIX_ENABLED_FUTURES", self.prev_futures.take());
        }
    }

    fn restore_env(key: &str, prev: Option<String>) {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn sort_query_params_ascending() {
        assert_eq!(sort_query_params(""), "");
        assert_eq!(sort_query_params("marginCoin=USDT"), "marginCoin=USDT");
        assert_eq!(sort_query_params("b=2&a=1"), "a=1&b=2");
    }

    #[tokio::test]
    async fn futures_signed_get_sends_header_auth() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("hdr");
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .and(query_param("marginCoin", "USDT"))
            .and(header_exists("api-key"))
            .and(header_exists("nonce"))
            .and(header_exists("timestamp"))
            .and(header_exists("sign"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0,
                "data": { "accountEquity": "100.5", "marginCoin": "USDT" }
            })))
            .mount(&server)
            .await;

        let connector = BitunixConnector::new(creds.config(&server.uri(), "http://unused", true));
        let body = connector
            .futures_signed_get("/api/v1/futures/account", "marginCoin=USDT")
            .await
            .expect("futures get");
        assert!(body["data"].is_object());
    }

    #[test]
    fn resolve_futures_account_array_shape() {
        let data = serde_json::json!([
            { "marginCoin": "USDT", "available": "250.0", "margin": "0" }
        ]);
        let account = resolve_futures_account(&data);
        assert_eq!(account["marginCoin"], "USDT");
        assert_eq!(account["available"], "250.0");
    }

    #[tokio::test]
    async fn sync_balances_futures_wallet_openapi_sample() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("bal_openapi");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [] }
            })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0,
                "msg": "success",
                "data": [
                    {
                        "marginCoin": "USDT",
                        "available": "1500.0",
                        "frozen": "100.0",
                        "margin": "200.0",
                        "crossUnrealizedPNL": "150.0",
                        "isolationUnrealizedPNL": "50.0"
                    }
                ]
            })))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "futures");
        assert_eq!(holdings[0].asset, "USDT");
        assert_eq!(holdings[0].quantity, 2000.0);
        assert_eq!(holdings[0].market_value_usd, Some(2000.0));
        assert_eq!(holdings[0].unrealized_pnl, Some(150.0));
    }

    #[tokio::test]
    async fn sync_balances_futures_wallet_empty_data_warns_no_row() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("bal_empty");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [] }
            })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0,
                "data": []
            })))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert!(holdings.is_empty());
    }

    #[tokio::test]
    async fn sync_balances_futures_wallet_code_nonzero_no_row() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("bal_code");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [] }
            })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 10001,
                "msg": "invalid signature",
                "data": [
                    { "marginCoin": "USDT", "accountEquity": "2500.0" }
                ]
            })))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert!(holdings.is_empty());
    }

    #[test]
    fn parse_futures_wallet_openapi_equity_fallback() {
        let connector = BitunixConnector::new(BitunixConfig {
            enabled: true,
            api_key_env: "unused".into(),
            api_secret_env: "unused".into(),
            spot_base_url: "http://unused".into(),
            futures_base_url: "http://unused".into(),
            enabled_futures: true,
        });
        let body = serde_json::json!({
            "code": 0,
            "data": [
                {
                    "marginCoin": "USDT",
                    "available": "1000",
                    "frozen": "0",
                    "margin": "500",
                    "crossUnrealizedPNL": "300",
                    "isolationUnrealizedPNL": "200"
                }
            ]
        });
        let holding = connector
            .parse_futures_wallet(&body)
            .expect("futures wallet row");
        assert_eq!(holding.product_type, "futures");
        assert_eq!(holding.quantity, 2000.0);
    }

    #[tokio::test]
    async fn sync_balances_futures_wallet_array_shape() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("bal_arr");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [] }
            })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    { "marginCoin": "USDT", "available": "180.0", "margin": "70.0", "frozen": "0" }
                ]
            })))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "futures");
        assert_eq!(holdings[0].asset, "USDT");
        assert_eq!(holdings[0].quantity, 250.0);
        assert_eq!(holdings[0].market_value_usd, Some(250.0));
    }

    #[tokio::test]
    async fn sync_balances_futures_wallet_product_type() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("bal");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [] }
            })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "accountEquity": "250.0", "marginCoin": "USDT", "unrealizedPnl": "12.5" }
            })))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "futures");
        assert_eq!(holdings[0].asset, "USDT");
        assert_eq!(holdings[0].quantity, 250.0);
        assert_eq!(holdings[0].market_value_usd, Some(250.0));
    }

    #[tokio::test]
    async fn sync_balances_spot_only_when_futures_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("spot");
        creds.disable_futures();
        let spot_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "balances": [{ "coin": "BTC", "available": "0.01" }] }
            })))
            .mount(&spot_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            "http://should-not-call",
            &spot_server.uri(),
            false,
        ));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_balances(&mut state)
            .await
            .expect("sync balances");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "spot");
        assert_eq!(holdings[0].asset, "BTC");
    }

    #[tokio::test]
    async fn sync_positions_linear_unrealized_pnl_casing() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("pos_pnl");
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/futures/position/get_pending_positions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "list": [
                        { "symbol": "INJUSDT", "qty": "1.5", "unrealizedPNL": "-3.25" }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let connector = BitunixConnector::new(creds.config(&server.uri(), "http://unused", true));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_positions(&mut state)
            .await
            .expect("sync positions");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "linear");
        assert_eq!(holdings[0].unrealized_pnl, Some(-3.25));
    }

    #[tokio::test]
    async fn sync_positions_linear_holdings() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("pos");
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/futures/position/get_pending_positions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "list": [
                        { "symbol": "BTCUSDT", "qty": "-0.5", "unrealizedPnl": "10.2" }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let connector = BitunixConnector::new(creds.config(&server.uri(), "http://unused", true));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_positions(&mut state)
            .await
            .expect("sync positions");

        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0].product_type, "linear");
        assert_eq!(holdings[0].asset, "BTCUSDT");
        assert_eq!(holdings[0].quantity, 0.5);
        assert!(holdings[0].market_value_usd.is_none());
        assert_eq!(state.active_symbols, vec!["BTCUSDT"]);
    }

    #[tokio::test]
    async fn sync_positions_empty_when_futures_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("pos_off");
        creds.disable_futures();
        let connector = BitunixConnector::new(creds.config("http://unused", "http://unused", false));
        let mut state = ExchangeSyncState::default();
        let holdings = connector
            .sync_positions(&mut state)
            .await
            .expect("sync positions");
        assert!(holdings.is_empty());
    }

    #[tokio::test]
    async fn test_connection_spot_ok_futures_fail_partial() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("tc_partial");
        let spot_server = MockServer::start().await;
        let futures_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": {} })))
            .mount(&spot_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/futures/account"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&futures_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            &futures_server.uri(),
            &spot_server.uri(),
            true,
        ));
        let result = connector.test_connection().await.expect("test");
        assert!(result.ok);
        assert!(result.message.contains("Spot: OK"));
        assert!(result.message.contains("Futures:"));
    }

    #[tokio::test]
    async fn test_connection_spot_fail() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("tc_fail");
        let spot_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(401).set_body_string("denied"))
            .mount(&spot_server)
            .await;

        let connector = BitunixConnector::new(creds.config("http://unused", &spot_server.uri(), true));
        let result = connector.test_connection().await.expect("test");
        assert!(!result.ok);
        assert!(result.message.starts_with("Spot:"));
    }

    #[tokio::test]
    async fn test_connection_spot_only_when_futures_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        let creds = TestCreds::new("tc_spot");
        creds.disable_futures();
        let spot_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/spot/v1/user/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": {} })))
            .mount(&spot_server)
            .await;

        let connector = BitunixConnector::new(creds.config(
            "http://should-not-call",
            &spot_server.uri(),
            false,
        ));
        let result = connector.test_connection().await.expect("test");
        assert!(result.ok);
        assert_eq!(result.message, "Spot balance read OK");
    }
}
