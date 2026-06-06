//! Same-origin reverse proxy to Grafana (DEC-0057). Mounted outside `/api/v1` JWT stack.

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        FromRequestParts, Request, State, WebSocketUpgrade,
    },
    http::{header::UPGRADE, HeaderMap, HeaderName, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest,
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message as WsMessage,
    },
};
use tracing::warn;
use url::Url;

const PREFIX: &str = "/analytics/grafana";

#[derive(Clone)]
pub struct GrafanaProxyState {
    pub upstream: Url,
    pub client: Client,
}

pub fn grafana_routes(upstream: Url) -> Router {
    let state = GrafanaProxyState {
        upstream,
        client: Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client"),
    };

    let inner = Router::new().fallback(any(proxy_entry));

    Router::new()
        .nest("/analytics/grafana", inner)
        .with_state(state)
}

pub fn upstream_path(request_path: &str) -> String {
    let rest = request_path
        .strip_prefix(PREFIX)
        .unwrap_or(request_path);
    if rest.is_empty() {
        "/".to_string()
    } else if rest.starts_with('/') {
        rest.to_string()
    } else {
        format!("/{rest}")
    }
}

fn build_upstream_url(base: &Url, path: &str, query: Option<&str>) -> Result<Url, ()> {
    let mut url = base.clone();
    url.set_path(path);
    url.set_query(query);
    Ok(url)
}

fn build_upstream_ws_url(base: &Url, path: &str, query: Option<&str>) -> Result<Url, ()> {
    let mut url = base.clone();
    let scheme = match url.scheme() {
        "https" => "wss",
        "http" => "ws",
        _ => return Err(()),
    };
    url.set_scheme(scheme).map_err(|_| ())?;
    url.set_path(path);
    url.set_query(query);
    Ok(url)
}

fn forward_request_header(name: &HeaderName) -> bool {
    !matches!(
        name.as_str(),
        "host" | "connection" | "upgrade" | "content-length" | "transfer-encoding"
    )
}

fn filter_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in headers.iter() {
        let n = name.as_str().to_ascii_lowercase();
        if n == "set-cookie" || n == "x-frame-options" {
            continue;
        }
        if n == "content-security-policy" {
            let v = value.to_str().unwrap_or("");
            if v.to_ascii_lowercase().contains("frame-ancestors") {
                continue;
            }
        }
        out.append(name.clone(), value.clone());
    }
    out
}

async fn proxy_entry(
    State(state): State<GrafanaProxyState>,
    req: Request,
) -> Response {
    if is_websocket_upgrade(req.headers()) && req.uri().path().contains("/api/live/") {
        let (mut parts, body) = req.into_parts();
        return match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
            Ok(ws) => {
                let req = Request::from_parts(parts, body);
                ws_live_proxy(ws, State(state), req).await.into_response()
            }
            Err(e) => e.into_response(),
        };
    }
    http_proxy(State(state), req).await
}

async fn http_proxy(
    State(state): State<GrafanaProxyState>,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = upstream_path(uri.path());
    let upstream_url = match build_upstream_url(&state.upstream, &path, uri.query()) {
        Ok(u) => u,
        Err(()) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let (parts, body) = req.into_parts();
    let mut builder = state.client.request(method, upstream_url);

    for (name, value) in &parts.headers {
        if forward_request_header(name) {
            builder = builder.header(name, value);
        }
    }

    let stream = body.into_data_stream();
    let body = reqwest::Body::wrap_stream(stream);

    let upstream_resp = match builder.body(body).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, "grafana proxy upstream request failed");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let status = upstream_resp.status();
    let headers = filter_response_headers(upstream_resp.headers());
    let body_stream = upstream_resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    let body = Body::from_stream(body_stream);

    let mut response = Response::new(body);
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

async fn ws_live_proxy(
    ws: WebSocketUpgrade,
    State(state): State<GrafanaProxyState>,
    req: Request,
) -> impl IntoResponse {
    let uri = req.uri().clone();
    let path = upstream_path(uri.path());
    let upstream_ws = match build_upstream_ws_url(&state.upstream, &path, uri.query()) {
        Ok(u) => u,
        Err(()) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let req_headers = req.headers().clone();
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = proxy_websocket(socket, upstream_ws, req_headers).await {
            warn!(error = %e, "grafana websocket proxy failed");
        }
    })
}

fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    headers
        .get(UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

async fn proxy_websocket(
    client_ws: WebSocket,
    upstream_url: Url,
    headers: HeaderMap,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = upstream_url.as_str().into_client_request()?;
    for (name, value) in &headers {
        if forward_request_header(name) {
            request.headers_mut().insert(name, value.clone());
        }
    }

    let (upstream_ws, _) = connect_async(request).await?;
    let (mut upstream_sink, mut upstream_stream) = upstream_ws.split();

    let (mut client_sink, mut client_stream) = client_ws.split();

    let client_to_upstream = async {
        while let Some(msg) = client_stream.next().await {
            match msg {
                Ok(Message::Text(t)) => {
                    upstream_sink
                        .send(WsMessage::Text(t))
                        .await?;
                }
                Ok(Message::Binary(b)) => {
                    upstream_sink
                        .send(WsMessage::Binary(b))
                        .await?;
                }
                Ok(Message::Ping(p)) => {
                    upstream_sink.send(WsMessage::Ping(p)).await?;
                }
                Ok(Message::Pong(p)) => {
                    upstream_sink.send(WsMessage::Pong(p)).await?;
                }
                Ok(Message::Close(frame)) => {
                    let frame = frame.map(|f| CloseFrame {
                        code: CloseCode::from(f.code),
                        reason: f.reason,
                    });
                    upstream_sink.send(WsMessage::Close(frame)).await?;
                    break;
                }
                Err(e) => return Err(Box::new(e) as _),
            }
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    let upstream_to_client = async {
        while let Some(msg) = upstream_stream.next().await {
            match msg? {
                WsMessage::Text(t) => client_sink.send(Message::Text(t)).await?,
                WsMessage::Binary(b) => client_sink.send(Message::Binary(b)).await?,
                WsMessage::Ping(p) => client_sink.send(Message::Ping(p)).await?,
                WsMessage::Pong(p) => client_sink.send(Message::Pong(p)).await?,
                WsMessage::Close(frame) => {
                    client_sink
                        .send(Message::Close(frame.map(|f| axum::extract::ws::CloseFrame {
                            code: f.code.into(),
                            reason: f.reason,
                        })))
                        .await?;
                    break;
                }
                WsMessage::Frame(_) => {}
            }
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    tokio::select! {
        r = client_to_upstream => r?,
        r = upstream_to_client => r?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_analytics_prefix() {
        assert_eq!(upstream_path("/analytics/grafana/"), "/");
        assert_eq!(upstream_path("/analytics/grafana"), "/");
        assert_eq!(
            upstream_path("/analytics/grafana/api/health"),
            "/api/health"
        );
        assert_eq!(
            upstream_path("/analytics/grafana/d/cashflow/cashflow"),
            "/d/cashflow/cashflow"
        );
    }
}
