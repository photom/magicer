# Magic Handlers Class Diagram

## Overview

HTTP handlers for file magic analysis endpoints (`/v1/magic/content` and `/v1/magic/path`).

## Class Diagram

```mermaid
classDiagram
    class MagicHandlers {
        <<module>>
        +analyze_content_handler() Handler
        +analyze_path_handler() Handler
    }
    
    class AnalyzeContentHandler {
        +State~AnalyzeContentUseCase~
        +Bytes
        +Query~FilenameQuery~
        +Result~Json~MagicResponse~, StatusCode~
    }
    
    class AnalyzePathHandler {
        +State~AnalyzePathUseCase~
        +Json~PathRequest~
        +Result~Json~MagicResponse~, StatusCode~
    }
    
    class FilenameQuery {
        +filename: String
    }
    
    class PathRequest {
        +relative_path: String
    }
    
    MagicHandlers --> AnalyzeContentHandler : contains
    MagicHandlers --> AnalyzePathHandler : contains
    
    note for MagicHandlers "HTTP handlers module\nAxum route handlers\nDomain-driven endpoints"
```

## analyze_content_handler

### Handler Signature

```rust
pub async fn analyze_content_handler(
    State(use_case): State<Arc<AnalyzeContentUseCase>>,
    Query(query): Query<FilenameQuery>,
    body: Bytes,
) -> Result<Json<MagicResponse>, (StatusCode, Json<ErrorResponse>)>
```

### Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Handler as analyze_content_handler
    participant Extractor as Axum Extractors
    participant UseCase as AnalyzeContentUseCase
    participant Response
    
    Client->>Handler: POST /v1/magic/content?filename=file.pdf<br/>Body: binary data
    Handler->>Extractor: Extract State, Query, Bytes
    Extractor-->>Handler: use_case, filename, body
    Handler->>Handler: Validate filename
    alt Invalid filename
        Handler-->>Client: 400 Bad Request
    else Valid filename
        Handler->>UseCase: execute(request)
        alt Use case succeeds
            UseCase-->>Response: Ok(MagicResponse)
            Handler-->>Client: 200 OK<br/>JSON response
        else Use case fails
            UseCase-->>Handler: Err(ApplicationError)
            Handler->>Handler: Map to HTTP status
            Handler-->>Client: 4xx/5xx Error<br/>JSON error response
        end
    end
```

### Request Example

```http
POST /v1/magic/content?filename=document.pdf HTTP/1.1
Host: api.example.com
Authorization: Basic YWRtaW46cGFzcw==
Content-Type: application/octet-stream
Content-Length: 12345

[binary PDF data...]
```

### Response Example

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "mime_type": "application/pdf",
  "description": "PDF document, version 1.4",
  "encoding": null,
  "analyzed_at": "2024-02-11T14:30:00Z"
}
```

### Error Handling

```mermaid
flowchart TD
    Handler[analyze_content_handler] --> CheckFilename{Filename<br/>valid?}
    CheckFilename -->|No| Err400[400 Bad Request<br/>Invalid filename]
    CheckFilename -->|Yes| CheckBody{Body<br/>empty?}
    CheckBody -->|Yes| Err400
    CheckBody -->|No| CallUseCase[use_case.execute]
    CallUseCase --> UseCaseResult{Result?}
    UseCaseResult -->|BadRequest| Err400
    UseCaseResult -->|UnprocessableEntity| Err422[422 Unprocessable Entity<br/>Analysis failed]
    UseCaseResult -->|InternalError| Err500[500 Internal Server Error<br/>System error]
    UseCaseResult -->|Ok| Success[200 OK<br/>JSON response]
    
    style Success fill:#90EE90
    style Err400 fill:#FFB6C1
    style Err422 fill:#FFEB3B
    style Err500 fill:#FF6B6B
```

---

## analyze_path_handler

### Handler Signature

```rust
pub async fn analyze_path_handler(
    State(use_case): State<Arc<AnalyzePathUseCase>>,
    Json(request): Json<PathRequest>,
) -> Result<Json<MagicResponse>, (StatusCode, Json<ErrorResponse>)>
```

### Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Handler as analyze_path_handler
    participant Extractor as Axum Extractors
    participant UseCase as AnalyzePathUseCase
    participant Response
    
    Client->>Handler: POST /v1/magic/path<br/>Body: {"relative_path": "docs/file.txt"}
    Handler->>Extractor: Extract State, Json
    Extractor-->>Handler: use_case, path_request
    Handler->>Handler: Validate path
    alt Invalid path
        Handler-->>Client: 400 Bad Request
    else Valid path
        Handler->>UseCase: execute(request)
        alt File found & analyzed
            UseCase-->>Response: Ok(MagicResponse)
            Handler-->>Client: 200 OK<br/>JSON response
        else File not found
            UseCase-->>Handler: Err(NotFound)
            Handler-->>Client: 404 Not Found
        else Path traversal
            UseCase-->>Handler: Err(Forbidden)
            Handler-->>Client: 403 Forbidden
        else Other error
            UseCase-->>Handler: Err(ApplicationError)
            Handler->>Handler: Map to HTTP status
            Handler-->>Client: 4xx/5xx Error
        end
    end
```

### Request Example

```http
POST /v1/magic/path HTTP/1.1
Host: api.example.com
Authorization: Basic YWRtaW46cGFzcw==
Content-Type: application/json

{
  "relative_path": "documents/report.pdf"
}
```

### Response Example

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "mime_type": "application/pdf",
  "description": "PDF document, version 1.7",
  "encoding": null,
  "analyzed_at": "2024-02-11T14:30:00Z"
}
```

### Error Responses

| HTTP Status | Error Code | Condition | Example Message |
|-------------|------------|-----------|-----------------|
| 400 Bad Request | `validation_error` | Invalid path format | `"Validation failed: Absolute path not allowed"` |
| 403 Forbidden | `access_denied` | Path traversal attempt | `"Access denied: Path outside sandbox"` |
| 404 Not Found | `not_found` | File doesn't exist | `"File not found: documents/report.pdf"` |
| 422 Unprocessable Entity | `processing_error` | Analysis failed | `"Cannot process: Unsupported file format"` |
| 500 Internal Server Error | `internal_error` | System error | `"Internal server error"` |

---

## Handler Implementation

```rust
pub async fn analyze_content_handler(
    State(use_case): State<Arc<AnalyzeContentUseCase>>,
    Query(query): Query<FilenameQuery>,
    body: Bytes,
) -> Result<Json<MagicResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate filename
    let filename = WindowsCompatibleFilename::new(query.filename)
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "validation_error",
                &format!("Invalid filename: {}", e),
            );
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;
    
    // Validate body
    if body.is_empty() {
        let error_response = ErrorResponse::new(
            "validation_error",
            "Request body is empty",
        );
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
    
    // Create request DTO
    let request = AnalyzeContentRequest::new(body, filename);
    
    // Execute use case
    use_case
        .execute(request)
        .map(Json)
        .map_err(|e| {
            let (status, error_response) = map_application_error(e);
            (status, Json(error_response))
        })
}

pub async fn analyze_path_handler(
    State(use_case): State<Arc<AnalyzePathUseCase>>,
    Json(path_request): Json<PathRequest>,
) -> Result<Json<MagicResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate and create RelativePath
    let relative_path = RelativePath::new(&path_request.relative_path)
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "validation_error",
                &format!("Invalid path: {}", e),
            );
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;
    
    // Create request DTO
    let request = AnalyzePathRequest::new(relative_path);
    
    // Execute use case
    use_case
        .execute(request)
        .map(Json)
        .map_err(|e| {
            let (status, error_response) = map_application_error(e);
            (status, Json(error_response))
        })
}
```

## Error Mapping

```rust
fn map_application_error(error: ApplicationError) -> (StatusCode, ErrorResponse) {
    match error {
        ApplicationError::BadRequest(msg) => (
            StatusCode::BAD_REQUEST,
            ErrorResponse::new("validation_error", &msg),
        ),
        ApplicationError::Unauthorized(msg) => (
            StatusCode::UNAUTHORIZED,
            ErrorResponse::new("authentication_required", &msg),
        ),
        ApplicationError::Forbidden(msg) => (
            StatusCode::FORBIDDEN,
            ErrorResponse::new("access_denied", &msg),
        ),
        ApplicationError::NotFound(msg) => (
            StatusCode::NOT_FOUND,
            ErrorResponse::new("not_found", &msg),
        ),
        ApplicationError::UnprocessableEntity(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorResponse::new("processing_error", &msg),
        ),
        ApplicationError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::new("internal_error", "Internal server error"),
        ),
        ApplicationError::Timeout => (
            StatusCode::GATEWAY_TIMEOUT,
            ErrorResponse::new("timeout", "Request timeout"),
        ),
    }
}
```

## Testing

```rust
#[tokio::test]
async fn test_analyze_content_success() {
    let app = create_test_app();
    
    let response = app
        .post("/v1/magic/content?filename=test.pdf")
        .body(b"PDF data")
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    let json: MagicResponse = response.json().await;
    assert_eq!(json.mime_type.as_str(), "application/pdf");
}

#[tokio::test]
async fn test_analyze_content_empty_body() {
    let app = create_test_app();
    
    let response = app
        .post("/v1/magic/content?filename=test.pdf")
        .body(b"")
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_analyze_path_success() {
    let app = create_test_app();
    
    let request = json!({
        "relative_path": "documents/report.pdf"
    });
    
    let response = app
        .post("/v1/magic/path")
        .json(&request)
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_analyze_path_not_found() {
    let app = create_test_app();
    
    let request = json!({
        "relative_path": "nonexistent/file.pdf"
    });
    
    let response = app
        .post("/v1/magic/path")
        .json(&request)
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_analyze_path_traversal() {
    let app = create_test_app();
    
    let request = json!({
        "relative_path": "../../../etc/passwd"
    });
    
    let response = app
        .post("/v1/magic/path")
        .json(&request)
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

## Design Rationale

- **Axum Extractors**: Leverage Axum's type-safe extractors for request parsing
- **DTO Boundary**: Convert HTTP types to application DTOs immediately
- **Error Mapping**: Translate application errors to HTTP status codes
- **Validation**: Validate inputs at handler level before reaching use cases
- **Type Safety**: Strong typing prevents runtime errors
- **Testability**: Easy to test with Axum test utilities
- **Separation**: Handlers orchestrate HTTP concerns, use cases handle business logic
