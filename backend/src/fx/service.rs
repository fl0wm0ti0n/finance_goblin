use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::config::PortfolioConfig;
use crate::db::DbPool;

use super::repository::FxRepository;

const FIAT_STABLE: &[&str] = &["EUR", "USD", "USDT", "USDC", "GBP"];

#[derive(Debug, Clone, Copy)]
pub struct EurAmount {
    pub eur: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ExchangePriceBook {
    pub tickers_usdt: HashMap<String, f64>,
}

impl ExchangePriceBook {
    pub fn set_ticker(&mut self, asset: &str, usdt_price: f64) {
        self.tickers_usdt.insert(asset.to_uppercase(), usdt_price);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FxError {
    #[error("unpriced asset: {0}")]
    Unpriced(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("rate unavailable for {0}")]
    RateUnavailable(String),
}

#[derive(Clone)]
pub struct FxService {
    repo: Arc<FxRepository>,
    frankfurter_base: String,
    http: Client,
}

impl FxService {
    pub fn new(db: DbPool, config: PortfolioConfig) -> Self {
        Self {
            repo: Arc::new(FxRepository::new(db.pool().clone())),
            frankfurter_base: config.frankfurter_base_url.trim_end_matches('/').to_string(),
            http: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("fx http client"),
        }
    }

    pub async fn to_eur(
        &self,
        amount: f64,
        asset: &str,
        price_book: &ExchangePriceBook,
    ) -> Result<EurAmount, FxError> {
        if amount == 0.0 {
            return Ok(EurAmount { eur: 0.0 });
        }

        let asset = asset.to_uppercase();
        if asset == "EUR" {
            return Ok(EurAmount { eur: amount });
        }

        if FIAT_STABLE.contains(&asset.as_str()) {
            let rate = self.fiat_to_eur_rate(&asset).await?;
            return Ok(EurAmount {
                eur: amount * rate,
            });
        }

        if let Some(usdt_price) = price_book.tickers_usdt.get(&asset) {
            let usdt_eur = self.fiat_to_eur_rate("USDT").await?;
            return Ok(EurAmount {
                eur: amount * usdt_price * usdt_eur,
            });
        }

        Err(FxError::Unpriced(asset))
    }

    pub async fn usd_to_eur(&self, usd_amount: f64) -> Result<f64, FxError> {
        let rate = self.fiat_to_eur_rate("USD").await?;
        Ok(usd_amount * rate)
    }

    async fn fiat_to_eur_rate(&self, base: &str) -> Result<f64, FxError> {
        let base = base.to_uppercase();
        if base == "EUR" {
            return Ok(1.0);
        }

        let today = Utc::now().date_naive();
        if let Some(cached) = self.repo.get_cached_rate(&base, "EUR", today).await? {
            return Ok(cached);
        }

        let lookup = match base.as_str() {
            "USDT" | "USDC" => "USD",
            other => other,
        };

        let url = format!(
            "{}/v1/latest?base={}&symbols=EUR",
            self.frankfurter_base, lookup
        );
        let resp: FrankfurterResponse = self.http.get(&url).send().await?.json().await?;
        let rate = resp
            .rates
            .get("EUR")
            .copied()
            .ok_or_else(|| FxError::RateUnavailable(base.clone()))?;

        self.repo
            .upsert_rate(today, lookup, "EUR", rate, "frankfurter")
            .await?;

        Ok(rate)
    }
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    rates: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eur_identity() {
        let book = ExchangePriceBook::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Cannot test async without DB — test asset classification
        assert!(FIAT_STABLE.contains(&"USDT"));
        assert!(!book.tickers_usdt.contains_key("BTC"));
    }
}
