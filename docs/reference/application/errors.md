# Application Errors Class Diagram

## Overview

`ApplicationError` represents all possible failures at the application layer, mapping domain errors to HTTP-friendly semantic errors.

## Class Diagram

```mermaid
classDiagram
    class ApplicationError {
        <<enumeration>>
        +BadRequest(String)
        +Unauthorized(String)
        +Forbidden(String)
        +NotFound(String)
        +UnprocessableEntity(String)
        +InternalError(String)
        +Timeout
    }
    
    class DomainError {
        <<enumeration>>
    }
    
    class StatusCode {
        <<external::http>>
    }
    
    ApplicationError ..> DomainError : maps from
    ApplicationError ..> StatusCode : maps to
    
    note for ApplicationError "Application layer errors\nHTTP semantic mapping\nDerives: Debug, Clone"
```

## Error Variants

| Variant | HTTP Status | Description | Use Case |
|---------|-------------|-------------|----------|
| `BadRequest(String)` | 400 | Invalid input or validation failure | Malformed request, invalid value object |
| `Unauthorized(String)` | 401 | Authentication required or failed | Missing or invalid credentials |
| `Forbidden(String)` | 403 | Authenticated but not authorized | Path traversal, sandbox violation |
| `NotFound(String)` | 404 | Resource not found | File doesn't exist |
| `UnprocessableEntity(String)` | 422 | Valid request but semantic error | File analysis failed, unsupported format |
| `InternalError(String)` | 500 | Unexpected system error | Configuration error, unexpected exception |
| `Timeout` | 504 | Request timeout | Analysis took too long |

## Error Mapping from Domain

```mermaid
graph TD
    Domain[DomainError] --> Map[Error Mapping]
    
    Map --> Validation[ValidationError]
    Validation --> BadRequest[ApplicationError::BadRequest]
    
    Map --> Magic[MagicError]
    Magic --> Unprocessable[ApplicationError::UnprocessableEntity]
    
    Map --> FileNotFound[FileNotFound]
    FileNotFound --> NotFound[ApplicationError::NotFound]
    
    Map --> Permission[PermissionDenied]
    Permission --> Forbidden[ApplicationError::Forbidden]
    
    Map --> Config[ConfigurationError]
    Config --> Internal[ApplicationError::InternalError]
    
    style BadRequest fill:#FFB6C1
    style Unprocessable fill:#FFEB3B
    style NotFound fill:#FFB6C1
    style Forbidden fill:#FFB6C1
    style Internal fill:#FF6B6B
```

## Mapping Implementation

```rust
impl From<DomainError> for ApplicationError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::ValidationError(e) => {
                ApplicationError::BadRequest(format!("Validation failed: {}", e))
            },
            DomainError::MagicError(e) => {
                ApplicationError::UnprocessableEntity(format!("Analysis failed: {}", e))
            },
            DomainError::FileNotFound(path) => {
                ApplicationError::NotFound(format!("File not found: {}", path))
            },
            DomainError::PermissionDenied(path) => {
                ApplicationError::Forbidden(format!("Access denied: {}", path))
            },
            DomainError::ConfigurationError(msg) => {
                ApplicationError::InternalError(format!("Configuration error: {}", msg))
            },
        }
    }
}
```

## HTTP Response Mapping

```mermaid
flowchart TD
    AppErr[ApplicationError] --> Map{Error variant?}
    
    Map -->|BadRequest| HTTP400["400 Bad Request<br/>Invalid input"]
    Map -->|Unauthorized| HTTP401["401 Unauthorized<br/>Authentication required"]
    Map -->|Forbidden| HTTP403["403 Forbidden<br/>Access denied"]
    Map -->|NotFound| HTTP404["404 Not Found<br/>Resource not found"]
    Map -->|UnprocessableEntity| HTTP422["422 Unprocessable Entity<br/>Semantic error"]
    Map -->|InternalError| HTTP500["500 Internal Server Error<br/>System error"]
    Map -->|Timeout| HTTP504["504 Gateway Timeout<br/>Request timeout"]
    
    style HTTP400 fill:#FFB6C1
    style HTTP401 fill:#FFB6C1
    style HTTP403 fill:#FFB6C1
    style HTTP404 fill:#FFB6C1
    style HTTP422 fill:#FFEB3B
    style HTTP500 fill:#FF6B6B
    style HTTP504 fill:#FFB6C1
```

## Error Response Format

```json
{
  "error": {
    "code": "validation_error",
    "message": "Validation failed: Filename too long (311, max 310)",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## Error Code Mapping

| ApplicationError | HTTP Status | Error Code | Message Template |
|------------------|-------------|------------|------------------|
| `BadRequest` | 400 | `validation_error` | "Validation failed: {details}" |
| `Unauthorized` | 401 | `authentication_required` | "Authentication required" |
| `Forbidden` | 403 | `access_denied` | "Access denied: {reason}" |
| `NotFound` | 404 | `not_found` | "Resource not found: {resource}" |
| `UnprocessableEntity` | 422 | `processing_error` | "Cannot process: {reason}" |
| `InternalError` | 500 | `internal_error` | "Internal server error" |
| `Timeout` | 504 | `timeout` | "Request timeout" |

## Usage Examples

### In Use Cases

```rust
impl<R: MagicRepository> AnalyzeContentUseCase<R> {
    pub fn execute(&self, request: AnalyzeContentRequest) -> Result<MagicResponse, ApplicationError> {
        // Validate request
        if request.content().is_empty() {
            return Err(ApplicationError::BadRequest("Content is empty".to_string()));
        }
        
        // Call repository
        let result = self.repository
            .analyze_buffer(request.content(), request.filename().as_str())
            .map_err(ApplicationError::from)?; // Automatic mapping
        
        // Map to response
        Ok(MagicResponse::from(result))
    }
}
```

### In HTTP Handlers

```rust
async fn analyze_content_handler(
    State(use_case): State<Arc<AnalyzeContentUseCase>>,
    Json(request): Json<AnalyzeContentRequest>,
) -> Result<Json<MagicResponse>, (StatusCode, Json<ErrorResponse>)> {
    use_case
        .execute(request)
        .map(Json)
        .map_err(|e| {
            let status = match &e {
                ApplicationError::BadRequest(_) => StatusCode::BAD_REQUEST,
                ApplicationError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                ApplicationError::Forbidden(_) => StatusCode::FORBIDDEN,
                ApplicationError::NotFound(_) => StatusCode::NOT_FOUND,
                ApplicationError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
                ApplicationError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                ApplicationError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            };
            
            let error_response = ErrorResponse {
                error: ErrorDetail {
                    code: e.code(),
                    message: e.to_string(),
                    request_id: RequestId::new(),
                },
            };
            
            (status, Json(error_response))
        })
}
```

## Error Context Flow

```mermaid
sequenceDiagram
    participant Handler
    participant UseCase
    participant Domain
    participant Infra
    
    Handler->>UseCase: execute(request)
    UseCase->>Domain: Validate value objects
    alt Validation fails
        Domain-->>UseCase: Err(ValidationError)
        UseCase->>UseCase: Map to BadRequest
        UseCase-->>Handler: Err(ApplicationError::BadRequest)
        Handler-->>Handler: Map to 400 response
    else Validation succeeds
        UseCase->>Infra: Call repository
        alt Infrastructure error
            Infra-->>Domain: Infrastructure error
            Domain->>Domain: Map to DomainError
            Domain-->>UseCase: Err(DomainError)
            UseCase->>UseCase: Map to ApplicationError
            UseCase-->>Handler: Err(ApplicationError)
            Handler-->>Handler: Map to HTTP status
        else Success
            Infra-->>Domain: Success
            Domain-->>UseCase: Ok(MagicResult)
            UseCase-->>Handler: Ok(MagicResponse)
            Handler-->>Handler: 200 OK
        end
    end
```

## Error Logging

```rust
impl ApplicationError {
    pub fn log(&self) {
        match self {
            ApplicationError::BadRequest(msg) => {
                log::warn!("Bad request: {}", msg);
            },
            ApplicationError::Unauthorized(msg) => {
                log::warn!("Unauthorized: {}", msg);
            },
            ApplicationError::Forbidden(msg) => {
                log::warn!("Forbidden: {}", msg);
            },
            ApplicationError::NotFound(msg) => {
                log::info!("Not found: {}", msg);
            },
            ApplicationError::UnprocessableEntity(msg) => {
                log::warn!("Unprocessable: {}", msg);
            },
            ApplicationError::InternalError(msg) => {
                log::error!("Internal error: {}", msg);
            },
            ApplicationError::Timeout => {
                log::warn!("Request timeout");
            },
        }
    }
}
```

## Error Metrics

| Error Type | Metric | Alert Threshold |
|------------|--------|-----------------|
| `BadRequest` | `http_errors_bad_request_total` | > 10% of requests |
| `Unauthorized` | `http_errors_unauthorized_total` | > 5% of requests |
| `Forbidden` | `http_errors_forbidden_total` | Investigate all |
| `NotFound` | `http_errors_not_found_total` | Monitor trend |
| `UnprocessableEntity` | `http_errors_unprocessable_total` | > 1% of requests |
| `InternalError` | `http_errors_internal_total` | > 0.1% of requests |
| `Timeout` | `http_errors_timeout_total` | > 0.5% of requests |

## Design Rationale

- **HTTP Semantics**: Error variants map directly to HTTP status codes
- **User-Friendly**: Include descriptive messages for debugging
- **Security**: Internal errors don't leak sensitive details
- **Traceability**: Include request ID in error responses
- **Layered**: Maps from domain errors without exposing domain internals
- **Actionable**: Error messages guide users to fix issues
