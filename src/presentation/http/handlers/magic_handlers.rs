use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    body::Body,
    Extension,
};
use crate::presentation::http::responses::magic_response::MagicResponse;
use crate::presentation::http::responses::error_response::ErrorResponse;
use crate::domain::value_objects::request_id::RequestId;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::presentation::state::app_state::AppState;
use std::sync::Arc;
use serde::Deserialize;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

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
    body: Body,
) -> impl IntoResponse {
    let mut body_stream = body.into_data_stream();
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

    let threshold = state.config.analysis.large_file_threshold_mb * 1024 * 1024;
    
    let mut buffer = Vec::new();
    let mut is_large = false;
    let mut temp_file_handler = None;
    let mut open_file = None;

    while let Some(chunk_result) = body_stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to read request body: {}", e),
                    request_id: Some(request_id.as_str().to_string()),
                }),
            ).into_response(),
        };

        if !is_large {
            if buffer.len() + chunk.len() > threshold {
                is_large = true;
                let temp_dir = std::path::Path::new(&state.config.analysis.temp_dir);
                let tf = match crate::infrastructure::filesystem::temp_file_handler::TempFileHandler::new_empty(temp_dir) {
                    Ok(f) => f,
                    Err(e) => return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to create temp file: {}", e),
                            request_id: Some(request_id.as_str().to_string()),
                        }),
                    ).into_response(),
                };
                
                let mut file = match tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(tf.path())
                    .await 
                {
                    Ok(f) => f,
                    Err(e) => return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to open temp file: {}", e),
                            request_id: Some(request_id.as_str().to_string()),
                        }),
                    ).into_response(),
                };
                
                if let Err(e) = file.write_all(&buffer).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to write buffer to temp file: {}", e),
                            request_id: Some(request_id.as_str().to_string()),
                        }),
                    ).into_response();
                }
                
                if let Err(e) = file.write_all(&chunk).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to write chunk to temp file: {}", e),
                            request_id: Some(request_id.as_str().to_string()),
                        }),
                    ).into_response();
                }
                
                temp_file_handler = Some(tf);
                open_file = Some(file);
                buffer.clear();
                buffer.shrink_to_fit();
            } else {
                buffer.extend_from_slice(&chunk);
            }
        } else {
            if let Some(ref mut file) = open_file {
                if let Err(e) = file.write_all(&chunk).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to write chunk to temp file: {}", e),
                            request_id: Some(request_id.as_str().to_string()),
                        }),
                    ).into_response();
                }
            }
        }
    }

    let result = if is_large {
        if let Some(file) = open_file.take() {
            if let Err(e) = file.sync_all().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to sync temp file: {}", e),
                        request_id: Some(request_id.as_str().to_string()),
                    }),
                ).into_response();
            }
            drop(file);
            
            let tf = temp_file_handler.as_ref().unwrap();
            state.analyze_content_use_case.execute_from_file(request_id.clone(), filename, tf.path()).await
        } else {
            unreachable!()
        }
    } else {
        state.analyze_content_use_case.execute(request_id.clone(), filename, &buffer).await
    };

    match result {
        Ok(res) => (StatusCode::OK, Json(MagicResponse::from(res))).into_response(),
        Err(e) => (
            e.status_code(),
            Json(ErrorResponse {
                error: format!("Analysis failed: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        )
            .into_response(),
    }
}

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
        Err(e) => (
            e.status_code(),
            Json(ErrorResponse {
                error: format!("Analysis failed: {}", e),
                request_id: Some(request_id.as_str().to_string()),
            }),
        )
            .into_response(),
    }
}
