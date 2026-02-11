use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    body::Bytes,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::presentation::state::app_state::AppState;
use crate::presentation::http::responses::magic_response::MagicResponse;
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
    body: Bytes,
) -> impl IntoResponse {
    let request_id = RequestId::generate();
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid filename").into_response(),
    };

    match state.analyze_content_use_case.execute(request_id, filename, &body).await {
        Ok(result) => (StatusCode::OK, Json(MagicResponse::from(result))).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn analyze_path(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyzePathQuery>,
) -> impl IntoResponse {
    let request_id = RequestId::generate();
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid filename").into_response(),
    };
    
    // We need to import RelativePath
    let path = match crate::domain::value_objects::path::RelativePath::new(&query.path) {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid path").into_response(),
    };

    match state.analyze_path_use_case.execute(request_id, filename, path).await {
        Ok(result) => (StatusCode::OK, Json(MagicResponse::from(result))).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
