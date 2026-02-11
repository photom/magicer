use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use crate::presentation::state::app_state::AppState;

pub async fn ping(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.health_check_use_case.execute().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
