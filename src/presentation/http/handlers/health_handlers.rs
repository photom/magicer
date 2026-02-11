use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Extension};
use std::sync::Arc;
use serde::Serialize;
use crate::presentation::state::app_state::AppState;
use crate::domain::value_objects::request_id::RequestId;

#[derive(Serialize)]
pub struct HealthResponse {
    pub message: String,
    pub request_id: String,
}

pub async fn ping(
    State(state): State<Arc<AppState>>,
    Extension(request_id): Extension<RequestId>,
) -> impl IntoResponse {
    match state.health_check_use_case.execute().await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                message: "pong".to_string(),
                request_id: request_id.as_str().to_string(),
            })
        ).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
