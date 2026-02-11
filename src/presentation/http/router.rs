use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use std::sync::Arc;
use crate::presentation::state::app_state::AppState;
use crate::presentation::http::handlers::{health_handlers, magic_handlers};
use crate::presentation::http::middleware::auth;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/content", post(magic_handlers::analyze_content))
        .route("/path", post(magic_handlers::analyze_path))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_auth))
        .with_state(state.clone());

    Router::new()
        .route("/v1/ping", get(health_handlers::ping))
        .nest("/v1/magic", api_routes)
        .with_state(state)
}
