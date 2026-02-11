use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use crate::fake_auth::FakeAuth;
use tower::ServiceExt; 
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::test]
async fn test_ping_handler() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let auth_service = Arc::new(FakeAuth);
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));
    let router = create_router(state);

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
}
