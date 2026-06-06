use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::ai::provider::{ChatCompletionMessage, ChatCompletionRequest, ProviderError};
use crate::AppState;

#[derive(Deserialize)]
pub struct AiTestRequest {
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

impl Default for AiTestRequest {
    fn default() -> Self {
        Self {
            prompt: default_prompt(),
        }
    }
}

fn default_prompt() -> String {
    "Reply OK.".into()
}

#[derive(Serialize)]
pub struct AiTestResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/api/v1/ai/test", axum::routing::post(ai_test))
}

pub async fn ai_test(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AiTestRequest>,
) -> Json<AiTestResponse> {
    let _user = headers
        .get("x-auth-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("authenticated");

    let prompt = body.prompt;
    let provider = state.ai.provider();
    let started = Instant::now();

    if !provider.is_configured() {
        return Json(AiTestResponse {
            ok: false,
            latency_ms: None,
            model: None,
            provider: provider.name().to_string(),
            sample: None,
            error: Some("provider not configured".into()),
        });
    }

    let req = ChatCompletionRequest {
        model: provider.model().to_string(),
        messages: vec![ChatCompletionMessage {
            role: "user".into(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }],
        tools: None,
        tool_choice: None,
        temperature: Some(
            state
                .ai
                .config
                .temperature
                .unwrap_or_else(|| provider.default_temperature()),
        ),
        stream: false,
        max_tokens: 16,
    };

    match provider.chat_completion(req).await {
        Ok(resp) => {
            let sample = resp
                .choices
                .first()
                .and_then(|c| c.message.content.clone())
                .unwrap_or_default();
            Json(AiTestResponse {
                ok: true,
                latency_ms: Some(started.elapsed().as_millis() as u64),
                model: Some(provider.model().to_string()),
                provider: provider.name().to_string(),
                sample: Some(sample.chars().take(200).collect()),
                error: None,
            })
        }
        Err(ProviderError::Request(msg)) | Err(ProviderError::Parse(msg)) => Json(AiTestResponse {
            ok: false,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            model: Some(provider.model().to_string()),
            provider: provider.name().to_string(),
            sample: None,
            error: Some(truncate_error(&msg)),
        }),
        Err(ProviderError::NotConfigured(env)) => Json(AiTestResponse {
            ok: false,
            latency_ms: None,
            model: None,
            provider: provider.name().to_string(),
            sample: None,
            error: Some(format!("set {env}")),
        }),
        Err(ProviderError::Config(msg)) => Json(AiTestResponse {
            ok: false,
            latency_ms: None,
            model: None,
            provider: provider.name().to_string(),
            sample: None,
            error: Some(msg),
        }),
    }
}

fn truncate_error(msg: &str) -> String {
    msg.chars().take(500).collect()
}

#[allow(dead_code)]
fn _status_ok() -> StatusCode {
    StatusCode::OK
}
