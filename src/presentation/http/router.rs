use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::presentation::state::app_state::AppState;
use crate::presentation::http::handlers::{health_handlers, magic_handlers};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/ping", get(health_handlers::ping))
        .route("/v1/magic/content", post(magic_handlers::analyze_content))
        .route("/v1/magic/path", post(magic_handlers::analyze_path))
        .with_state(state)
}
