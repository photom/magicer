use crate::domain::value_objects::request_id::RequestId;
use crate::presentation::http::responses::error_response::ErrorResponse;
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

pub async fn handle_error(request: Request, next: Next) -> Response {
    let request_id = request.extensions().get::<RequestId>().cloned();
    let response = next.run(request).await;

    if response.status().is_success() {
        return response;
    }

    let status = response.status();

    // If the response is already JSON, we assume it's already formatted correctly.
    // Otherwise, we wrap it in our standard ErrorResponse.
    let content_type = response.headers().get(axum::http::header::CONTENT_TYPE);
    let is_json = content_type.map_or(false, |v| {
        v.to_str().map_or(false, |s| s.contains("application/json"))
    });

    if is_json {
        return response;
    }

    // Convert non-JSON error response to ErrorResponse
    let error_message = status
        .canonical_reason()
        .unwrap_or("Unknown error")
        .to_string();

    (
        status,
        Json(ErrorResponse {
            error: error_message,
            request_id: request_id.map(|id| id.as_str().to_string()),
        }),
    )
        .into_response()
}
