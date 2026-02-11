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
async fn test_analyze_content_handler_success() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let auth_service = Arc::new(FakeAuth);
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));
    let router = create_router(state);

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
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));
    let router = create_router(state);

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
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert_ne!(body_str, "Not implemented");
}
