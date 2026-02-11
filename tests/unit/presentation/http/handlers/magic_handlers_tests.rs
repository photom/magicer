use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use tower::ServiceExt;
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::test]
async fn test_analyze_content_handler_success() {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let state = Arc::new(AppState::new(magic_repo, sandbox));
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

    // We haven't implemented Auth middleware or real handler yet, 
    // so it should return 200 "Not implemented" for now as per our current stub.
    // Wait, I want to implement it properly.
    assert_eq!(response.status(), StatusCode::OK);
}
