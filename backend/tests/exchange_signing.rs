//! Exchange HTTP signing tests with wiremock.

use flow_finance_ai::exchanges::http::{bitunix_sign, hmac_sha256_hex, ExchangeHttpClient};
use reqwest::Method;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn get_only_audit_rejects_write_methods() {
    assert!(ExchangeHttpClient::audit_method(&Method::POST).is_err());
    assert!(ExchangeHttpClient::audit_method(&Method::GET).is_ok());
}

#[test]
fn binance_hmac_signing_fixture() {
    let sig = hmac_sha256_hex("testsecret", "symbol=BTCUSDT&timestamp=1234567890");
    assert_eq!(sig.len(), 64);
    assert_ne!(sig, hmac_sha256_hex("other", "symbol=BTCUSDT&timestamp=1234567890"));
}

#[test]
fn bitunix_double_sha256_fixture() {
    let sig = bitunix_sign("testsecret", "timestamp=1234567890");
    assert_eq!(sig.len(), 64);
    assert_eq!(sig, bitunix_sign("testsecret", "timestamp=1234567890"));
}

#[tokio::test]
async fn exchange_http_get_with_mock_server() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let client = ExchangeHttpClient::new();
    let url = format!("{}/api/v3/ping", server.uri());
    let body = client.get_with_backoff(&url, vec![]).await.expect("get");
    assert!(body.is_object());
}
