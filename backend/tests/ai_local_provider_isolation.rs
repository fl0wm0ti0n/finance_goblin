//! AC5: local provider must never call api.openai.com (wiremock guard).

use flow_finance_ai::ai::provider::{build_provider, ChatCompletionMessage, ChatCompletionRequest};
use flow_finance_ai::config::AiConfig;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn local_config(base: &str) -> AiConfig {
    AiConfig {
        provider: "ollama".into(),
        base_url: base.into(),
        model: "test-model".into(),
        api_key_env: "OPENAI_API_KEY".into(),
        temperature: None,
        local_tool_nudge_retry: false,
        max_tool_rounds: 2,
        max_completion_tokens: 256,
        max_tool_result_bytes: 8192,
        request_timeout_secs: 10,
        rate_limit_per_min: 100,
        audit_retention_days: 90,
        audit_max_rows: 500,
    }
}

#[tokio::test]
async fn local_provider_never_calls_openai_com() {
    std::env::remove_var("OPENAI_API_KEY");

    let local = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{
                "message": { "role": "assistant", "content": "OK" },
                "finish_reason": "stop"
            }]
        })))
        .mount(&local)
        .await;

    let base = format!("{}/v1", local.uri());
    let cfg = local_config(&base);
    let provider = build_provider(&cfg).expect("build local provider");

    assert_eq!(provider.name(), "ollama");
    assert!(provider.is_local());
    assert!(provider.omit_tool_choice());

    let resp = provider
        .chat_completion(ChatCompletionRequest {
            model: cfg.model.clone(),
            messages: vec![ChatCompletionMessage {
                role: "user".into(),
                content: Some("ping".into()),
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
        .expect("local mock should respond");

    assert_eq!(
        resp.choices[0].message.content.as_deref(),
        Some("OK")
    );

    let hits = local.received_requests().await.expect("requests");
    assert_eq!(hits.len(), 1);
    assert!(hits[0].url.as_str().contains("/v1/chat/completions"));
    assert!(!hits[0].url.as_str().contains("api.openai.com"));
}

#[test]
fn openai_mode_includes_tool_choice_and_cloud_defaults() {
    std::env::set_var("OPENAI_API_KEY", "test-key");
    let mut cfg = local_config("");
    cfg.provider = "openai".into();
    let provider = build_provider(&cfg).unwrap();
    assert!(!provider.omit_tool_choice());
    assert!(!provider.is_local());
    assert_eq!(provider.display_label(), "Cloud · OpenAI");
    std::env::remove_var("OPENAI_API_KEY");
}
