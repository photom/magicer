use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

pub async fn handle_error(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status().is_success() {
        return response;
    }
    
    let status = response.status();
    
    if status.is_client_error() || status.is_server_error() {
        return response;
    }

    response
}
