use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::ai::audit::{AuditInsert, AuditRepository};
use crate::ai::privacy::PrivacyLayer;
use crate::ai::provider::{
    AiProvider, ChatCompletionMessage, ChatCompletionRequest, ProviderError, ToolCall,
};
use crate::ai::registry::ToolRegistry;
use crate::ai::types::{
    truncate_tool_result, ChatMessage, ChatRequest, SseDoneEvent, SseErrorEvent, SseTokenEvent,
    ToolContext,
};
use crate::config::AiConfig;

const SYSTEM_PROMPT: &str = "You are a privacy-safe financial assistant. Use tools for factual balances and spending. Never invent numbers. Prefer aggregates when raw transactions are disabled.

When listing subscriptions or cancelable services, enumerate every display_name from get_subscriptions results with amounts — never generic industry-only lists when tool data exists.
For utility/merchant/category keywords (e.g. Strom, Amazon, streaming), use get_transactions with category_search — never pass keywords as category_id.
When period_status is no_rows_in_period, state mirror_date_bounds from the tool response so the operator knows whether the period is outside synced data.
get_subscriptions status/kind accept only documented enum values — never privacy hashes (Counterparty-*).";

const LOCAL_TOOL_NUDGE: &str =
    "Use the available tools to answer with factual data from the user's finances. Call a tool when the question needs balances, spending, or projections.";

#[derive(Debug, Clone, Serialize)]
pub struct SseWarningEvent {
    pub code: String,
    pub message: String,
}

pub struct CompletionOutcome {
    pub content: String,
    pub tools_used: Vec<String>,
    pub warnings: Vec<SseWarningEvent>,
}

pub struct AiOrchestrator {
    config: AiConfig,
    registry: Arc<ToolRegistry>,
    privacy: Arc<PrivacyLayer>,
    audit: Arc<AuditRepository>,
}

impl AiOrchestrator {
    pub fn new(
        config: AiConfig,
        registry: Arc<ToolRegistry>,
        privacy: Arc<PrivacyLayer>,
        audit: Arc<AuditRepository>,
    ) -> Self {
        Self {
            config,
            registry,
            privacy,
            audit,
        }
    }

    pub async fn complete(
        &self,
        provider: &dyn AiProvider,
        ctx: &ToolContext,
        request: ChatRequest,
    ) -> Result<CompletionOutcome, ProviderError> {
        let mut messages = self.build_messages(&request.messages);
        let tools = self.registry.build_openai_tools();
        let mut tools_used = Vec::new();
        let mut warnings = Vec::new();
        let mut nudge_used = false;

        for _round in 0..self.config.max_tool_rounds {
            let response = provider
                .chat_completion(self.build_provider_request(provider, &messages, &tools, false))
                .await?;

            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| ProviderError::Parse("empty choices".into()))?;

            if choice.finish_reason.as_deref() == Some("tool_calls")
                || choice.message.tool_calls.is_some()
            {
                messages.push(choice.message.clone());
                let tool_calls = choice.message.tool_calls.clone().unwrap_or_default();
                for call in tool_calls {
                    let result = self
                        .execute_tool_call(provider, ctx, &call, &mut tools_used)
                        .await?;
                    messages.push(ChatCompletionMessage {
                        role: "tool".into(),
                        content: Some(result),
                        tool_calls: None,
                        tool_call_id: Some(call.id.clone()),
                        name: Some(call.function.name.clone()),
                    });
                }
                continue;
            }

            let content = choice.message.content.clone().unwrap_or_default();
            if provider.is_local()
                && !content.is_empty()
                && self.config.local_tool_nudge_retry
                && !nudge_used
            {
                nudge_used = true;
                messages.push(choice.message.clone());
                messages.push(ChatCompletionMessage {
                    role: "user".into(),
                    content: Some(LOCAL_TOOL_NUDGE.into()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
                continue;
            }

            if should_warn_local_no_tools(provider.is_local(), &tools_used, &content) {
                warnings.push(SseWarningEvent {
                    code: "local_no_tools".into(),
                    message: "Local model answered without calling tools; results may be less accurate."
                        .into(),
                });
            }

            return Ok(CompletionOutcome {
                content,
                tools_used,
                warnings,
            });
        }

        Ok(CompletionOutcome {
            content: "Reached maximum tool rounds.".into(),
            tools_used,
            warnings,
        })
    }

    pub fn spawn_stream(
        self: Arc<Self>,
        provider: Arc<dyn AiProvider>,
        ctx: ToolContext,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> mpsc::Receiver<Result<String, String>> {
        let (tx, rx) = mpsc::channel(64);
        tokio::spawn(async move {
            if let Err(e) = self
                .run_stream(provider.as_ref(), &ctx, request, &tx, cancel)
                .await
            {
                let _ = tx
                    .send(Err(format_event(
                        "error",
                        &serde_json::to_string(&SseErrorEvent {
                            code: "provider_error".into(),
                            message: e.to_string(),
                        })
                        .unwrap_or_default(),
                    )))
                    .await;
            }
        });
        rx
    }

    async fn run_stream(
        &self,
        provider: &dyn AiProvider,
        ctx: &ToolContext,
        request: ChatRequest,
        tx: &mpsc::Sender<Result<String, String>>,
        cancel: CancellationToken,
    ) -> Result<(), ProviderError> {
        let outcome = self.complete(provider, ctx, request).await?;
        if cancel.is_cancelled() {
            return Ok(());
        }

        for warning in &outcome.warnings {
            tx.send(Ok(format_event(
                "warning",
                &serde_json::to_string(warning).unwrap_or_default(),
            )))
            .await
            .map_err(|_| ProviderError::Request("client disconnected".into()))?;
        }

        for chunk in outcome.content.split_inclusive(' ') {
            if cancel.is_cancelled() {
                return Ok(());
            }
            let event = SseTokenEvent {
                delta: chunk.to_string(),
            };
            tx.send(Ok(format_event(
                "token",
                &serde_json::to_string(&event).unwrap_or_default(),
            )))
            .await
            .map_err(|_| ProviderError::Request("client disconnected".into()))?;
        }

        let done = SseDoneEvent {
            message_id: Uuid::new_v4().to_string(),
            tools_used: outcome.tools_used,
        };
        tx.send(Ok(format_event(
            "done",
            &serde_json::to_string(&done).unwrap_or_default(),
        )))
        .await
        .map_err(|_| ProviderError::Request("client disconnected".into()))?;
        Ok(())
    }

    fn build_provider_request(
        &self,
        provider: &dyn AiProvider,
        messages: &[ChatCompletionMessage],
        tools: &[Value],
        stream: bool,
    ) -> ChatCompletionRequest {
        let mut req = ChatCompletionRequest {
            model: provider.model().to_string(),
            messages: messages.to_vec(),
            tools: Some(tools.to_vec()),
            tool_choice: None,
            temperature: Some(
                self.config
                    .temperature
                    .unwrap_or_else(|| provider.default_temperature()),
            ),
            stream,
            max_tokens: provider.max_completion_tokens(),
        };
        if !provider.omit_tool_choice() {
            req.tool_choice = Some("auto".into());
        }
        req
    }

    async fn execute_tool_call(
        &self,
        provider: &dyn AiProvider,
        ctx: &ToolContext,
        call: &ToolCall,
        tools_used: &mut Vec<String>,
    ) -> Result<String, ProviderError> {
        let started = Instant::now();
        let tool_name = &call.function.name;
        tools_used.push(tool_name.clone());

        let args: Value = serde_json::from_str(&call.function.arguments)
            .unwrap_or_else(|_| Value::Object(Default::default()));

        let args_summary = self.privacy.summarize_args(&args);

        let (status, result_value, err_msg, result_rows) =
            match self.registry.execute(tool_name, ctx, args).await {
            Ok(mut value) => {
                let rows = audit_result_rows(tool_name, &value);
                value = self.privacy.redact_tool_result(tool_name, value);
                value = truncate_tool_result(value, self.config.max_tool_result_bytes);
                ("ok".to_string(), value, None, rows)
            }
            Err(e) => (
                "error".into(),
                Value::Object(serde_json::Map::from_iter([(
                    "error".into(),
                    Value::String(e.to_string()),
                )])),
                Some(e.to_string()),
                None,
            ),
        };

        let duration_ms = started.elapsed().as_millis() as i32;
        let _ = self
            .audit
            .insert(AuditInsert {
                session_id: ctx.session_id,
                user_subject: ctx.user_subject.clone(),
                tool_name: tool_name.clone(),
                args_summary,
                result_status: status.clone(),
                result_rows,
                duration_ms,
                error_message: err_msg,
                model: Some(provider.model().to_string()),
                provider: Some(provider.name().to_string()),
            })
            .await;

        serde_json::to_string(&result_value).map_err(|e| ProviderError::Parse(e.to_string()))
    }

    fn build_messages(&self, user_messages: &[ChatMessage]) -> Vec<ChatCompletionMessage> {
        let mut messages = vec![ChatCompletionMessage {
            role: "system".into(),
            content: Some(SYSTEM_PROMPT.into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        for m in user_messages {
            messages.push(ChatCompletionMessage {
                role: m.role.clone(),
                content: Some(m.content.clone()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }
        messages
    }
}

pub fn format_event(event: &str, data: &str) -> String {
    format!("event: {event}\ndata: {data}\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use sqlx::PgPool;

    struct StubProvider {
        local: bool,
        temp: f32,
    }

    #[async_trait]
    impl AiProvider for StubProvider {
        fn name(&self) -> &str {
            "stub"
        }
        fn is_configured(&self) -> bool {
            true
        }
        fn is_local(&self) -> bool {
            self.local
        }
        fn display_label(&self) -> &str {
            "Stub"
        }
        fn omit_tool_choice(&self) -> bool {
            self.local
        }
        fn default_temperature(&self) -> f32 {
            self.temp
        }
        fn model(&self) -> &str {
            "m"
        }
        fn max_completion_tokens(&self) -> u32 {
            64
        }
        fn base_url(&self) -> &str {
            "http://stub/v1"
        }
        async fn chat_completion(
            &self,
            _req: ChatCompletionRequest,
        ) -> Result<crate::ai::provider::ChatCompletionResponse, ProviderError> {
            Err(ProviderError::Request("stub".into()))
        }
        async fn chat_completion_stream(
            &self,
            _req: ChatCompletionRequest,
        ) -> Result<reqwest::Response, ProviderError> {
            Err(ProviderError::Request("stub".into()))
        }
    }

    #[test]
    fn sse_event_format() {
        let s = format_event("token", r#"{"delta":"hi"}"#);
        assert!(s.starts_with("event: token"));
        assert!(s.contains("data:"));
    }

    #[tokio::test]
    async fn request_builder_omits_tool_choice_for_local() {
        let orch = AiOrchestrator::new(
            AiConfig::default(),
            Arc::new(ToolRegistry::build()),
            Arc::new(PrivacyLayer::new(Default::default())),
            Arc::new(AuditRepository::new(
                PgPool::connect_lazy("postgres://u:p@localhost/db").unwrap(),
            )),
        );
        let local = StubProvider {
            local: true,
            temp: 0.3,
        };
        let req = orch.build_provider_request(&local, &[], &[], false);
        assert!(req.tool_choice.is_none());
        assert_eq!(req.temperature, Some(0.3));

        let cloud = StubProvider {
            local: false,
            temp: 0.7,
        };
        let req = orch.build_provider_request(&cloud, &[], &[], false);
        assert_eq!(req.tool_choice.as_deref(), Some("auto"));
    }

    #[test]
    fn local_no_tools_warning_predicate() {
        assert!(should_warn_local_no_tools(true, &[], "answer"));
        assert!(!should_warn_local_no_tools(false, &[], "answer"));
        assert!(!should_warn_local_no_tools(true, &["get_portfolio".into()], "answer"));
    }

    #[test]
    fn audit_result_rows_for_transactions_and_subscriptions() {
        let tx = serde_json::json!({
            "by_category": [{"category_id": "1"}, {"category_id": "2"}],
            "total_transaction_count": 10
        });
        assert_eq!(audit_result_rows("get_transactions", &tx), Some(2));

        let tx_no_buckets = serde_json::json!({"total_transaction_count": 5});
        assert_eq!(audit_result_rows("get_transactions", &tx_no_buckets), Some(5));

        let subs = serde_json::json!({"patterns_count": 7});
        assert_eq!(audit_result_rows("get_subscriptions", &subs), Some(7));

        assert_eq!(audit_result_rows("get_forecast", &subs), None);

        let err = serde_json::json!({"error": "fail"});
        assert_eq!(audit_result_rows("get_transactions", &err), None);
    }

    #[test]
    fn system_prompt_includes_discovery_rules() {
        assert!(SYSTEM_PROMPT.contains("category_search"));
        assert!(SYSTEM_PROMPT.contains("display_name"));
        assert!(SYSTEM_PROMPT.contains("mirror_date_bounds"));
        assert!(SYSTEM_PROMPT.contains("Counterparty-*"));
    }
}

fn should_warn_local_no_tools(is_local: bool, tools_used: &[String], content: &str) -> bool {
    is_local && tools_used.is_empty() && !content.is_empty()
}

fn audit_result_rows(tool_name: &str, result: &Value) -> Option<i32> {
    if result.get("error").is_some() {
        return None;
    }
    match tool_name {
        "get_transactions" => result
            .get("by_category")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as i32)
            .or_else(|| {
                result
                    .get("total_transaction_count")
                    .and_then(|v| v.as_i64())
                    .map(|n| n as i32)
            }),
        "get_subscriptions" => result
            .get("patterns_count")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32),
        _ => None,
    }
}
