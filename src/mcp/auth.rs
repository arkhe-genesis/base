//! Cathedral ARKHE v28.3.2 — MCP Auth Middleware
//! Autenticação via Bearer Token.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub struct AuthConfig {
    pub required_token: String,
}

pub async fn auth_middleware(
    axum::extract::State(config): axum::extract::State<Arc<AuthConfig>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if config.required_token.is_empty() {
        return Ok(next.run(req).await);
    }

    // Extrai o cabeçalho Authorization
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // após "Bearer "
    if token != config.required_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}