use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AiConfig;

const OPENAI_DEFAULT_BASE: &str = "https://api.openai.com/v1";
const OLLAMA_DEFAULT_BASE: &str = "http://ollama:11434/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProviderKind {
    OpenAi,
    Ollama,
    OpenAiCompatible,
}

impl AiProviderKind {
    pub fn parse(s: &str) -> Result<Self, ProviderError> {
        match s.trim().to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAi),
            "ollama" => Ok(Self::Ollama),
            "openai_compatible" => Ok(Self::OpenAiCompatible),
            other => Err(ProviderError::Config(format!(
                "invalid ai.provider '{other}': expected openai, ollama, or openai_compatible"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Ollama => "ollama",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider not configured: set {0} env var")]
    NotConfigured(String),
    #[error("provider config error: {0}")]
    Config(String),
    #[error("openai request failed: {0}")]
    Request(String),
    #[error("parse error: {0}")]
    Parse(String),
}

/// Trait for chat completion backends (OpenAI, Ollama, OpenAI-compatible).
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;
    fn is_local(&self) -> bool;
    fn display_label(&self) -> &str;
    fn omit_tool_choice(&self) -> bool;
    fn default_temperature(&self) -> f32;
    fn model(&self) -> &str;
    fn max_completion_tokens(&self) -> u32;
    fn base_url(&self) -> &str;

    async fn chat_completion(
        &self,
        req: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError>;

    async fn chat_completion_stream(
        &self,
        req: ChatCompletionRequest,
    ) -> Result<reqwest::Response, ProviderError>;
}

pub struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
    max_completion_tokens: u32,
    kind: AiProviderKind,
}

impl OpenAiCompatibleProvider {
    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }

    fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        match self.api_key.as_ref().filter(|k| !k.is_empty()) {
            Some(key) => builder.bearer_auth(key),
            None => builder,
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        self.kind.as_str()
    }

    fn is_configured(&self) -> bool {
        match self.kind {
            AiProviderKind::OpenAi => self.api_key.as_ref().is_some_and(|k| !k.is_empty()),
            AiProviderKind::Ollama => true,
            AiProviderKind::OpenAiCompatible => !self.base_url.is_empty(),
        }
    }

    fn is_local(&self) -> bool {
        matches!(
            self.kind,
            AiProviderKind::Ollama | AiProviderKind::OpenAiCompatible
        )
    }

    fn display_label(&self) -> &str {
        match self.kind {
            AiProviderKind::OpenAi => "Cloud · OpenAI",
            AiProviderKind::Ollama => "Local · Ollama",
            AiProviderKind::OpenAiCompatible => "Local · Compatible",
        }
    }

    fn omit_tool_choice(&self) -> bool {
        self.is_local()
    }

    fn default_temperature(&self) -> f32 {
        if self.is_local() {
            0.3
        } else {
            0.7
        }
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn max_completion_tokens(&self) -> u32 {
        self.max_completion_tokens
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError> {
        let resp = self
            .apply_auth(
                self.client
                    .post(self.chat_url())
                    .json(&request),
            )
            .send()
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Request(body));
        }

        resp.json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))
    }

    async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<reqwest::Response, ProviderError> {
        let resp = self
            .apply_auth(
                self.client
                    .post(self.chat_url())
                    .json(&request),
            )
            .send()
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Request(body));
        }

        Ok(resp)
    }
}

pub fn build_provider(config: &AiConfig) -> Result<Arc<dyn AiProvider>, ProviderError> {
    let kind = AiProviderKind::parse(&config.provider)?;
    let api_key = config.api_key().map(|s| s.to_string());

    let (base_url, configured_check) = match kind {
        AiProviderKind::OpenAi => {
            let key = api_key.clone().filter(|k| !k.is_empty());
            if key.is_none() {
                return Err(ProviderError::NotConfigured(config.api_key_env.clone()));
            }
            (OPENAI_DEFAULT_BASE.to_string(), true)
        }
        AiProviderKind::Ollama => {
            let url = config
                .effective_base_url()
                .unwrap_or_else(|| OLLAMA_DEFAULT_BASE.to_string());
            (url, true)
        }
        AiProviderKind::OpenAiCompatible => {
            let url = config.effective_base_url().unwrap_or_default();
            if url.is_empty() {
                return Err(ProviderError::Config(
                    "openai_compatible requires non-empty [ai] base_url".into(),
                ));
            }
            (url, true)
        }
    };

    if !configured_check {
        return Err(ProviderError::Config("provider not configured".into()));
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .build()
        .map_err(|e| ProviderError::Request(e.to_string()))?;

    let provider = OpenAiCompatibleProvider {
        client,
        base_url,
        api_key,
        model: config.model.clone(),
        max_completion_tokens: config.max_completion_tokens,
        kind,
    };

    tracing::info!(
        provider = provider.name(),
        label = provider.display_label(),
        base_url = %provider.base_url(),
        model = %provider.model(),
        configured = provider.is_configured(),
        "AI provider initialized"
    );

    Ok(Arc::new(provider))
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatCompletionMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct StreamDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
pub struct StreamToolCallDelta {
    pub index: usize,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<StreamFunctionDelta>,
}

#[derive(Debug, Deserialize)]
pub struct StreamFunctionDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(provider: &str, base_url: &str) -> AiConfig {
        AiConfig {
            provider: provider.into(),
            base_url: base_url.into(),
            model: "test-model".into(),
            api_key_env: "OPENAI_API_KEY".into(),
            temperature: None,
            local_tool_nudge_retry: true,
            max_tool_rounds: 5,
            max_completion_tokens: 64,
            max_tool_result_bytes: 8192,
            request_timeout_secs: 5,
            rate_limit_per_min: 20,
            audit_retention_days: 90,
            audit_max_rows: 500,
        }
    }

    #[test]
    fn kind_parse_and_labels() {
        assert_eq!(
            AiProviderKind::parse("ollama").unwrap(),
            AiProviderKind::Ollama
        );
        assert!(AiProviderKind::parse("invalid").is_err());

        let cfg = test_config("ollama", "");
        std::env::remove_var("OPENAI_API_KEY");
        let p = build_provider(&cfg).unwrap();
        assert_eq!(p.display_label(), "Local · Ollama");
        assert!(p.omit_tool_choice());
        assert!(p.is_local());
        assert!(p.is_configured());
    }

    #[test]
    fn openai_requires_api_key() {
        std::env::remove_var("OPENAI_API_KEY");
        let cfg = test_config("openai", "");
        assert!(matches!(
            build_provider(&cfg),
            Err(ProviderError::NotConfigured(_))
        ));
    }

    #[test]
    fn openai_compatible_requires_base_url() {
        let cfg = test_config("openai_compatible", "");
        assert!(matches!(build_provider(&cfg), Err(ProviderError::Config(_))));
    }

    #[tokio::test]
    async fn http_posts_to_configured_base_without_bearer_when_no_key() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "OK" },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&server)
            .await;

        let mut cfg = test_config("openai_compatible", &format!("{}/v1", server.uri()));
        std::env::remove_var("OPENAI_API_KEY");
        let p = build_provider(&cfg).unwrap();
        let resp = p
            .chat_completion(ChatCompletionRequest {
                model: "m".into(),
                messages: vec![ChatCompletionMessage {
                    role: "user".into(),
                    content: Some("hi".into()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                tools: None,
                tool_choice: None,
                temperature: None,
                stream: false,
                max_tokens: 8,
            })
            .await
            .unwrap();
        assert_eq!(
            resp.choices[0].message.content.as_deref(),
            Some("OK")
        );
        cfg.provider = "openai".into();
    }

    #[tokio::test]
    async fn http_maps_401_to_request_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let cfg = test_config("openai_compatible", &format!("{}/v1", server.uri()));
        let p = build_provider(&cfg).unwrap();
        let err = p
            .chat_completion(ChatCompletionRequest {
                model: "m".into(),
                messages: vec![ChatCompletionMessage {
                    role: "user".into(),
                    content: Some("x".into()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                tools: None,
                tool_choice: None,
                temperature: None,
                stream: false,
                max_tokens: 8,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ProviderError::Request(_)));
    }
}
