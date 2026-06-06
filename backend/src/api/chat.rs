use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use futures::StreamExt;
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::ai::orchestrator::format_event;
use crate::ai::provider::ProviderError;
use crate::ai::types::{ChatRequest, SseErrorEvent};
use crate::AppState;

#[derive(Serialize)]
pub struct CompletionResponse {
    pub content: String,
    pub tools_used: Vec<String>,
    pub session_id: String,
}

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/api/v1/chat/stream", axum::routing::post(chat_stream))
        .route("/api/v1/chat/completions", axum::routing::post(chat_completions))
}

pub async fn chat_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let user = headers
        .get("x-auth-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("authenticated")
        .to_string();

    if !state.ai.check_rate_limit(&user).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate_limit_exceeded" })),
        ));
    }

    let provider = state.ai.provider();
    if !provider.is_configured() {
        let err = SseErrorEvent {
            code: "provider_not_configured".into(),
            message: format!(
                "AI provider '{}' is not configured — check config.toml and env, then restart",
                provider.name()
            ),
        };
        let chunk = format_event(
            "error",
            &serde_json::to_string(&err).unwrap_or_default(),
        );
        let stream = futures::stream::once(async move { Ok(chunk) });
        return Ok(sse_response(stream));
    }

    let session_id = body.session_id.unwrap_or_else(Uuid::new_v4);
    let ctx = state.ai.tool_context(&user, session_id);
    let cancel = CancellationToken::new();
    let cancel_child = cancel.child_token();

    let rx = state
        .ai
        .orchestrator
        .clone()
        .spawn_stream(provider, ctx, body, cancel_child);

    let stream = async_stream::stream! {
        let mut keepalive = tokio::time::interval(Duration::from_secs(15));
        keepalive.tick().await;
        let mut rx = rx;
        loop {
            tokio::select! {
                _ = keepalive.tick() => {
                    yield Ok::<_, std::convert::Infallible>(format!(": keepalive\n\n"));
                }
                msg = rx.recv() => {
                    match msg {
                        Some(Ok(chunk)) => yield Ok(chunk),
                        Some(Err(e)) => {
                            let err = SseErrorEvent { code: "stream_error".into(), message: e };
                            yield Ok(format_event("error", &serde_json::to_string(&err).unwrap_or_default()));
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    };

    Ok(sse_response(stream))
}

fn sse_response<S>(stream: S) -> Response
where
    S: futures::Stream<Item = Result<String, std::convert::Infallible>> + Send + 'static,
{
    let body = Body::from_stream(stream.map(|r| r.map(axum::body::Bytes::from)));
    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("X-Accel-Buffering", "no")
        .body(body)
        .unwrap()
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Json<CompletionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = headers
        .get("x-auth-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("authenticated")
        .to_string();

    if !state.ai.check_rate_limit(&user).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate_limit_exceeded" })),
        ));
    }

    let provider = state.ai.provider();
    if !provider.is_configured() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "provider_not_configured",
                "provider": provider.name()
            })),
        ));
    }

    let session_id = body.session_id.unwrap_or_else(Uuid::new_v4);
    let ctx = state.ai.tool_context(&user, session_id);

    match state
        .ai
        .orchestrator
        .complete(provider.as_ref(), &ctx, body)
        .await
    {
        Ok(outcome) => Ok(Json(CompletionResponse {
            content: outcome.content,
            tools_used: outcome.tools_used,
            session_id: session_id.to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}
