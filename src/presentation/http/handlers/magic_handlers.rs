use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    body::Bytes,
    Extension,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::presentation::state::app_state::AppState;
use crate::presentation::http::responses::magic_response::MagicResponse;
use crate::presentation::http::responses::error_response::ErrorResponse;
use crate::domain::value_objects::request_id::RequestId;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;

#[derive(Deserialize)]
pub struct AnalyzeQuery {
    pub filename: String,
}

#[derive(Deserialize)]
pub struct AnalyzePathQuery {
    pub filename: String,
    pub path: String,
}

pub async fn analyze_content(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyzeQuery>,
    Extension(request_id): Extension<RequestId>,
    body: Bytes,
) -> impl IntoResponse {
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid filename: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        ).into_response(),
    };

    match state.analyze_content_use_case.execute(request_id.clone(), filename, &body).await {
        Ok(result) => (StatusCode::OK, Json(MagicResponse::from(result))).into_response(),
        Err(e) => (
            e.status_code(),
            Json(ErrorResponse {
                error: format!("Analysis failed: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        ).into_response(),
    }
}

pub async fn analyze_path(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyzePathQuery>,
    Extension(request_id): Extension<RequestId>,
) -> impl IntoResponse {
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid filename: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        ).into_response(),
    };
    
    let path = match crate::domain::value_objects::path::RelativePath::new(&query.path) {
        Ok(p) => p,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid path: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        ).into_response(),
    };

    match state.analyze_path_use_case.execute(request_id.clone(), filename, path).await {
        Ok(result) => (StatusCode::OK, Json(MagicResponse::from(result))).into_response(),
        Err(e) => (
            e.status_code(),
            Json(ErrorResponse {
                error: format!("Analysis failed: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        ).into_response(),
    }
}
