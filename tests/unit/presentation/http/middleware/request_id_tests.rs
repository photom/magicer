use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
    routing::get,
    Router,
};
use magicer::presentation::http::middleware::request_id::add_request_id;
use magicer::domain::value_objects::request_id::RequestId;
use tower::ServiceExt;

#[tokio::test]
async fn test_request_id_middleware() {
    let app = Router::new()
        .route("/", get(|req: Request<Body>| async move {
            let request_id = req.extensions().get::<RequestId>();
            assert!(request_id.is_some());
            StatusCode::OK
        }))
        .layer(from_fn(add_request_id));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
}

#[tokio::test]
async fn test_request_id_respects_existing_header() {
    let existing_id = RequestId::generate();
    let app = Router::new()
        .route("/", get({
            let existing_id = existing_id.clone();
            move |req: Request<Body>| async move {
                let request_id = req.extensions().get::<RequestId>().unwrap();
                assert_eq!(request_id.as_str(), existing_id.as_str());
                StatusCode::OK
            }
        }))
        .layer(from_fn(add_request_id));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-request-id", existing_id.as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-request-id").unwrap().to_str().unwrap(),
        existing_id.as_str()
    );
}
