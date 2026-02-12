use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use magicer::presentation::http::middleware::{error_handler, request_id};
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use crate::fake_auth::FakeAuth;
use tower::ServiceExt;
use std::sync::Arc;
use std::path::PathBuf;
use axum::middleware;

#[tokio::test]
async fn test_analyze_content_handler_success() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let auth_service = Arc::new(FakeAuth);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service, config));
    let router = create_router(state)
        .layer(middleware::from_fn(error_handler::handle_error))
        .layer(middleware::from_fn(request_id::add_request_id));

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/magic/content?filename=test.pdf")
                .header("Authorization", "Basic YWRtaW46c2VjcmV0") // admin:secret
                .body(Body::from("%PDF-1.4"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.get("result").is_some());
}

#[tokio::test]
async fn test_analyze_path_handler_success() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let temp_dir = "/tmp/magicer_unit_handlers";
    std::fs::create_dir_all(temp_dir).unwrap();
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(temp_dir)));
    
    // Create file
    std::fs::write(PathBuf::from(temp_dir).join("test.pdf"), b"%PDF-1.4").unwrap();

    let auth_service = Arc::new(FakeAuth);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service, config));
    let router = create_router(state)
        .layer(middleware::from_fn(error_handler::handle_error))
        .layer(middleware::from_fn(request_id::add_request_id));

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/magic/path?filename=test.pdf&path=test.pdf")
                .header("Authorization", "Basic YWRtaW46c2VjcmV0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["result"]["mime_type"], "application/pdf");
}
