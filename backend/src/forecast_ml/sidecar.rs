use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ForecastMlConfig;

#[derive(Debug, Clone, Serialize)]
pub struct SidecarPoint {
    pub ds: String,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForecastRequest {
    pub series_id: String,
    pub freq: String,
    pub points: Vec<SidecarPoint>,
    pub horizon: u32,
    pub level: Vec<u32>,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastPoint {
    pub ds: String,
    pub y: f64,
    pub y_lo: f64,
    pub y_hi: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastResponse {
    pub model_family: String,
    pub seasonal_periods: Vec<u32>,
    pub seasonal_strength: f64,
    pub seasonal_detected: bool,
    pub forecasts: Vec<ForecastPoint>,
    pub backtest_wmape: Option<f64>,
    pub low_confidence: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    #[error("sidecar unavailable")]
    Unavailable,
    #[error("sidecar timeout")]
    Timeout,
    #[error("sidecar error: {0}")]
    Http(String),
    #[error("insufficient history")]
    InsufficientHistory,
}

#[derive(Clone)]
pub struct SidecarClient {
    client: Client,
    base_url: String,
}

impl SidecarClient {
    pub fn new(config: &ForecastMlConfig) -> Self {
        let timeout = Duration::from_secs(config.sidecar_timeout_secs);
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: config.sidecar_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn health_ok(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn forecast(&self, req: &ForecastRequest) -> Result<ForecastResponse, SidecarError> {
        let url = format!("{}/v1/forecast", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SidecarError::Timeout
                } else {
                    SidecarError::Unavailable
                }
            })?;

        if resp.status().as_u16() == 422 {
            return Err(SidecarError::InsufficientHistory);
        }
        if !resp.status().is_success() {
            return Err(SidecarError::Http(format!("status {}", resp.status())));
        }

        resp.json::<ForecastResponse>()
            .await
            .map_err(|e| SidecarError::Http(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn sidecar_forecast_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model_family": "AutoETS",
                "seasonal_periods": [12],
                "seasonal_strength": 0.4,
                "seasonal_detected": true,
                "forecasts": [{"ds": "2025-01-01", "y": 100.0, "y_lo": 80.0, "y_hi": 120.0}],
                "backtest_wmape": 0.1,
                "low_confidence": false
            })))
            .mount(&server)
            .await;

        let cfg = ForecastMlConfig {
            sidecar_url: server.uri(),
            sidecar_timeout_secs: 5,
            ..Default::default()
        };
        let client = SidecarClient::new(&cfg);
        let resp = client
            .forecast(&ForecastRequest {
                series_id: "test".into(),
                freq: "MS".into(),
                points: vec![SidecarPoint {
                    ds: "2024-01-01".into(),
                    y: 100.0,
                }],
                horizon: 1,
                level: vec![90],
                model: "auto".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.model_family, "AutoETS");
    }

    #[tokio::test]
    async fn sidecar_unavailable() {
        let cfg = ForecastMlConfig {
            sidecar_url: "http://127.0.0.1:1".into(),
            sidecar_timeout_secs: 1,
            ..Default::default()
        };
        let client = SidecarClient::new(&cfg);
        assert!(!client.health_ok().await);
    }
}
