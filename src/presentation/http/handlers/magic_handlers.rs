use crate::application::errors::ApplicationError;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::presentation::http::responses::error_response::ErrorResponse;
use crate::presentation::http::responses::magic_response::MagicResponse;
use crate::presentation::state::app_state::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use opentelemetry::KeyValue;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;

#[derive(Deserialize, Debug)]
pub struct AnalyzeQuery {
    pub filename: String,
}

#[derive(Deserialize, Debug)]
pub struct AnalyzePathQuery {
    pub filename: String,
    pub path: String,
}

/// Map an [`ApplicationError`] to the `error.kind` string value as defined in
/// `docs/reference/OBSERVABILITY.md` §7.2.
fn error_kind(e: &ApplicationError) -> &'static str {
    match e {
        ApplicationError::Timeout => "timeout",
        ApplicationError::BadRequest(_) => "bad_request",
        ApplicationError::NotFound(_) => "not_found",
        ApplicationError::InternalError(_) | ApplicationError::UnprocessableEntity(_) => "internal",
        ApplicationError::InsufficientStorage(_) => "insufficient_storage",
        ApplicationError::Unauthorized(_) => "unauthorized",
        ApplicationError::Forbidden(_) => "forbidden",
    }
}

#[tracing::instrument(
    name = "handler.analyze_content",
    fields(
        analysis.filename = %query.filename,
        analysis.strategy = tracing::field::Empty,
        error.kind = tracing::field::Empty,
    ),
    skip(state, headers, body, request_id, query),
)]
pub async fn analyze_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AnalyzeQuery>,
    Extension(request_id): Extension<RequestId>,
    body: Body,
) -> impl IntoResponse {
    let is_chunked = headers
        .get(axum::http::header::TRANSFER_ENCODING)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("chunked"))
        .unwrap_or(false);

    let content_length = headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let threshold = (state.config.analysis.large_file_threshold_mb * 1024 * 1024) as u64;

    let force_to_file = is_chunked || content_length.map(|l| l > threshold).unwrap_or(false);

    let body_stream = body.into_data_stream();
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid filename: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            )
                .into_response()
        }
    };

    let strategy_str = if force_to_file { "temp_file" } else { "in_memory" };
    tracing::Span::current().record("analysis.strategy", strategy_str);

    // Track active requests
    let active_labels = [
        KeyValue::new("http.method", "POST"),
        KeyValue::new("http.route", "/v1/magic/content"),
    ];
    state.metrics.http_active_requests.add(1, &active_labels);

    let start = Instant::now();

    let result = if force_to_file {
        state
            .analyze_content_use_case
            .analyze_to_temp_file(request_id.clone(), filename, body_stream)
            .await
    } else {
        state
            .analyze_content_use_case
            .analyze_in_memory(request_id.clone(), filename, body_stream)
            .await
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let analysis_type = if force_to_file {
        "content_to_file"
    } else {
        "content_in_memory"
    };

    state.metrics.http_active_requests.add(-1, &active_labels);

    match result {
        Ok(res) => {
            state.metrics.analysis_duration.record(
                elapsed_ms,
                &[KeyValue::new("analysis.type", analysis_type)],
            );
            (StatusCode::OK, Json(MagicResponse::from(res))).into_response()
        }
        Err(e) => {
            let kind = error_kind(&e);
            tracing::Span::current().record("error.kind", kind);
            state
                .metrics
                .analysis_errors
                .add(1, &[KeyValue::new("error.kind", kind)]);
            (
                e.status_code(),
                Json(ErrorResponse {
                    error: format!("Analysis failed: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            )
                .into_response()
        }
    }
}

#[tracing::instrument(
    name = "handler.analyze_path",
    fields(
        analysis.filename = %query.filename,
        error.kind = tracing::field::Empty,
    ),
    skip(state, request_id, query),
)]
pub async fn analyze_path(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyzePathQuery>,
    Extension(request_id): Extension<RequestId>,
) -> impl IntoResponse {
    let filename = match WindowsCompatibleFilename::new(&query.filename) {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid filename: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            )
                .into_response()
        }
    };

    let path = match crate::domain::value_objects::path::RelativePath::new(&query.path) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid path: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            )
                .into_response()
        }
    };

    match state
        .analyze_path_use_case
        .execute(request_id.clone(), filename, path)
        .await
    {
        Ok(result) => (StatusCode::OK, Json(MagicResponse::from(result))).into_response(),
        Err(e) => {
            let kind = error_kind(&e);
            tracing::Span::current().record("error.kind", kind);
            state
                .metrics
                .analysis_errors
                .add(1, &[KeyValue::new("error.kind", kind)]);
            (
                e.status_code(),
                Json(ErrorResponse {
                    error: format!("Analysis failed: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            )
                .into_response()
        }
    }
}
