use axum::{extract::Request, middleware::Next, response::Response};
use crate::domain::value_objects::request_id::RequestId;

pub async fn add_request_id(mut request: Request, next: Next) -> Response {
    let request_id = RequestId::generate();
    request.extensions_mut().insert(request_id.clone());
    
    let mut response = next.run(request).await;
    
    // Also add to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.as_str().parse().unwrap(),
    );
    
    response
}
