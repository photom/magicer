use crate::domain::value_objects::auth::BasicAuthCredentials;
use crate::presentation::state::app_state::AppState;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Basic ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let credentials = auth_header.trim_start_matches("Basic ");
    let decoded = general_purpose::STANDARD
        .decode(credentials)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let decoded_str = String::from_utf8(decoded).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let credentials =
        BasicAuthCredentials::new(parts[0], parts[1]).map_err(|_| StatusCode::UNAUTHORIZED)?;

    state
        .auth_service
        .verify_credentials(&credentials)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}
