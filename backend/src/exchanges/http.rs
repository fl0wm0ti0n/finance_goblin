use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use serde_json::Value;
use tracing::warn;

use super::types::ExchangeError;

const MAX_RETRIES: u32 = 4;

pub struct ExchangeHttpClient {
    client: Client,
}

impl ExchangeHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("exchange http client"),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn audit_method(method: &Method) -> Result<(), ExchangeError> {
        if *method != Method::GET {
            return Err(ExchangeError::MethodNotAllowed(method.to_string()));
        }
        Ok(())
    }

    pub async fn get_with_backoff(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> Result<Value, ExchangeError> {
        Self::audit_method(&Method::GET)?;

        let mut attempt = 0u32;
        loop {
            let mut req = self.client.get(url);
            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }

            let resp = req.send().await?;
            let status = resp.status();

            if status == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                warn!(url, attempt, ?delay, "exchange rate limited; backing off");
                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(ExchangeError::Api(format!("HTTP {status}: {body}")));
            }

            return Ok(resp.json().await?);
        }
    }
}

impl Default for ExchangeHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hmac_sha256_hex(secret: &str, message: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn bitunix_sign(secret: &str, message: &str) -> String {
    use sha2::{Digest, Sha256};
    let first = Sha256::digest(message.as_bytes());
    let second_input = format!("{}{}", hex::encode(first), secret);
    hex::encode(Sha256::digest(second_input.as_bytes()))
}

/// Bitunix futures private REST sign per official spec (DEC-0062 / R-0058).
pub fn bitunix_futures_sign(
    secret: &str,
    nonce: &str,
    timestamp_ms: &str,
    api_key: &str,
    query_params: &str,
    body: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let digest_input = format!("{nonce}{timestamp_ms}{api_key}{query_params}{body}");
    let digest = hex::encode(Sha256::digest(digest_input.as_bytes()));
    let sign_input = format!("{digest}{secret}");
    hex::encode(Sha256::digest(sign_input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_only_audit_rejects_post() {
        assert!(ExchangeHttpClient::audit_method(&Method::POST).is_err());
        assert!(ExchangeHttpClient::audit_method(&Method::PUT).is_err());
        assert!(ExchangeHttpClient::audit_method(&Method::DELETE).is_err());
        assert!(ExchangeHttpClient::audit_method(&Method::GET).is_ok());
    }

    #[test]
    fn hmac_sha256_deterministic() {
        let sig = hmac_sha256_hex("secret", "query");
        assert_eq!(sig.len(), 64);
        assert_eq!(hmac_sha256_hex("secret", "query"), sig);
    }

    #[test]
    fn bitunix_double_sha256() {
        let sig = bitunix_sign("secret", "payload");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn bitunix_futures_sign_matches_official_fixture() {
        let body = r#"{"uid":"2899","arr":[{"id":1,"name":"maple"},{"id":2,"name":"lily"}]}"#;
        let sig = bitunix_futures_sign(
            "yourSecretKey",
            "123456",
            "20241120123045",
            "yourApiKey",
            "id1uid200",
            body,
        );
        assert_eq!(sig, "00397cd1e52c7dce3258067324363b6361fabc9178a0912b330c138db8745655");
    }

    #[test]
    fn bitunix_spot_sign_unchanged_regression() {
        let sig = bitunix_sign("testsecret", "timestamp=1234567890");
        assert_eq!(sig.len(), 64);
        assert_eq!(sig, bitunix_sign("testsecret", "timestamp=1234567890"));
    }
}
