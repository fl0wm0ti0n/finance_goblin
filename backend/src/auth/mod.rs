use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    aud: Option<serde_json::Value>,
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.config.oidc.dev_bypass {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    validate_token(&state, token).await?;
    req.extensions_mut().insert(AuthUser {
        subject: "authenticated".into(),
    });
    Ok(next.run(req).await)
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub subject: String,
}

async fn validate_token(state: &AppState, token: &str) -> Result<(), StatusCode> {
    let issuer = state.config.oidc.issuer_url.trim();
    if issuer.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Dev/test: accept unsigned tokens when issuer contains "test"
    if issuer.contains("test") {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.set_audience(&[state.config.oidc.audience.as_str()]);
        let key = DecodingKey::from_secret(b"test-secret");
        decode::<Claims>(token, &key, &validation).map_err(|_| StatusCode::UNAUTHORIZED)?;
        return Ok(());
    }

    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let alg = header.alg;

    let jwks_url = format!(
        "{}/.well-known/jwks.json",
        issuer.trim_end_matches('/')
    );
    let jwks: serde_json::Value = reqwest::get(&jwks_url)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .json()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;
    let keys = jwks["keys"]
        .as_array()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let jwk = keys
        .iter()
        .find(|k| k["kid"].as_str() == Some(&kid))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let n = jwk["n"].as_str().ok_or(StatusCode::UNAUTHORIZED)?;
    let e = jwk["e"].as_str().ok_or(StatusCode::UNAUTHORIZED)?;

    let mut validation = Validation::new(alg);
    validation.set_issuer(&[issuer]);
    if !state.config.oidc.audience.is_empty() {
        validation.set_audience(&[state.config.oidc.audience.as_str()]);
    }

    let key = DecodingKey::from_rsa_components(n, e).map_err(|_| StatusCode::UNAUTHORIZED)?;
    decode::<Claims>(token, &key, &validation).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(())
}
