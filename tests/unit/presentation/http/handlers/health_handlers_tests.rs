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
use crate::fake_temp_storage::FakeTempStorageService;
use tower::ServiceExt;
 
use std::sync::Arc;
use std::path::PathBuf;
use axum::middleware;

#[tokio::test]
async fn test_ping_handler() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let temp_storage = Arc::new(FakeTempStorageService::new(PathBuf::from("/tmp")));
    let auth_service = Arc::new(FakeAuth);
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let state = Arc::new(AppState::new(magic_repo, sandbox, temp_storage, auth_service, config));
    let router = create_router(state)
        .layer(middleware::from_fn(error_handler::handle_error))
        .layer(middleware::from_fn(request_id::add_request_id));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/v1/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["message"], "pong");
    assert!(json.get("request_id").is_some());
}
