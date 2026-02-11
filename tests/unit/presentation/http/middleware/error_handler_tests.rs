use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
    routing::get,
    Router,
    response::IntoResponse,
};
use magicer::presentation::http::middleware::error_handler::handle_error;
use magicer::presentation::http::middleware::request_id::add_request_id;
use tower::ServiceExt;

#[tokio::test]
async fn test_error_handler_wraps_non_json_errors() {
    let app = Router::new()
        .route("/error", get(|| async { StatusCode::BAD_REQUEST }))
        .layer(from_fn(handle_error))
        .layer(from_fn(add_request_id));

    let response = app
        .oneshot(Request::builder().uri("/error").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(json["error"], "Bad Request");
    assert!(json.get("request_id").is_some());
}

#[tokio::test]
async fn test_error_handler_ignores_success() {
    let app = Router::new()
        .route("/success", get(|| async { StatusCode::OK }))
        .layer(from_fn(handle_error));

    let response = app
        .oneshot(Request::builder().uri("/success").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap().len(), 0);
}
