use axum::{extract::Request, middleware::Next, response::Response};
use crate::domain::value_objects::request_id::RequestId;

pub async fn add_request_id(mut request: Request, next: Next) -> Response {
    let request_id = if let Some(header_val) = request.headers().get("x-request-id") {
        if let Ok(header_str) = header_val.to_str() {
            RequestId::parse(header_str).unwrap_or_else(|_| RequestId::generate())
        } else {
            RequestId::generate()
        }
    } else {
        RequestId::generate()
    };

    request.extensions_mut().insert(request_id.clone());
    
    let mut response = next.run(request).await;
    
    // Also add to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.as_str().parse().unwrap(),
    );
    
    response
}
