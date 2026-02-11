# Middleware Class Diagrams

## Overview

Axum middleware layers for cross-cutting concerns: request ID injection, authentication, timeout enforcement, and error handling.

## Middleware Architecture

```mermaid
graph TD
    Request[HTTP Request] --> RequestId[Request ID Middleware]
    RequestId --> Timeout[Timeout Middleware]
    Timeout --> Auth[Auth Middleware]
    Auth --> ErrorHandler[Error Handler Middleware]
    ErrorHandler --> Handler[Route Handler]
    Handler --> Response[HTTP Response]
    
    style RequestId fill:#E3F2FD
    style Timeout fill:#FFF3E0
    style Auth fill:#E8F5E9
    style ErrorHandler fill:#F3E5F5
```

---

## Request ID Middleware

### Class Diagram

```mermaid
classDiagram
    class RequestIdMiddleware {
        +request_id_layer() Layer
        +extract_or_generate_request_id(req: &Request) RequestId
        +inject_request_id(req: &mut Request, id: RequestId)
    }
    
    class RequestId {
        <<value object>>
    }
    
    RequestIdMiddleware ..> RequestId : uses
    
    note for RequestIdMiddleware "Generates UUID v4\nInjects into request extensions\nAdds X-Request-ID header to response"
```

### Flow

```mermaid
sequenceDiagram
    participant Client
    participant Middleware as RequestIdMiddleware
    participant Extensions
    participant Handler
    
    Client->>Middleware: HTTP Request
    Middleware->>Middleware: Check X-Request-ID header
    alt Header present
        Middleware->>Middleware: Parse UUID
        alt Valid UUID
            Middleware->>Extensions: Store existing ID
        else Invalid UUID
            Middleware->>Middleware: Generate new UUID
            Middleware->>Extensions: Store new ID
        end
    else Header absent
        Middleware->>Middleware: Generate new UUID
        Middleware->>Extensions: Store new ID
    end
    Middleware->>Handler: Forward request
    Handler-->>Middleware: Response
    Middleware->>Middleware: Add X-Request-ID header
    Middleware-->>Client: Response with X-Request-ID
```

### Implementation

```rust
pub fn request_id_layer() -> ServiceBuilder<Stack<RequestIdLayer, Identity>> {
    ServiceBuilder::new()
        .layer(RequestIdLayer)
}

pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdMiddleware<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        RequestIdMiddleware { inner }
    }
}

pub struct RequestIdMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RequestIdMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = /* ... */;
    
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        // Extract or generate request ID
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| RequestId::parse(s).ok())
            .unwrap_or_else(RequestId::new);
        
        // Inject into request extensions
        req.extensions_mut().insert(request_id.clone());
        
        // Continue to next layer
        let fut = self.inner.call(req);
        
        async move {
            let mut res = fut.await?;
            
            // Add X-Request-ID header to response
            res.headers_mut().insert(
                "x-request-id",
                request_id.to_string().parse().unwrap(),
            );
            
            Ok(res)
        }
    }
}
```

---

## Authentication Middleware

### Class Diagram

```mermaid
classDiagram
    class AuthMiddleware {
        +auth_layer(service: Arc~AuthenticationService~) Layer
        +extract_basic_auth(req: &Request) Result~(String, String), AuthError~
    }
    
    class AuthenticationService {
        <<trait>>
    }
    
    class AuthError {
        <<enumeration>>
        MissingHeader
        InvalidFormat
        InvalidCredentials
        InternalError
    }
    
    AuthMiddleware ..> AuthenticationService : uses
    AuthMiddleware ..> AuthError : returns
    
    note for AuthMiddleware "HTTP Basic Auth\nConstant-time verification\nPublic endpoints bypass"
```

### Flow

```mermaid
sequenceDiagram
    participant Client
    participant Middleware as AuthMiddleware
    participant Service as AuthenticationService
    participant Handler
    
    Client->>Middleware: HTTP Request
    Middleware->>Middleware: Check if public endpoint
    alt Public endpoint (e.g., /v1/ping)
        Middleware->>Handler: Forward without auth check
        Handler-->>Client: Response
    else Protected endpoint
        Middleware->>Middleware: Extract Authorization header
        alt Header missing
            Middleware-->>Client: 401 Unauthorized
        else Header present
            Middleware->>Middleware: Parse Basic auth
            alt Invalid format
                Middleware-->>Client: 401 Unauthorized
            else Valid format
                Middleware->>Service: verify_credentials
                alt Valid credentials
                    Service-->>Middleware: Ok(true)
                    Middleware->>Handler: Forward request
                    Handler-->>Client: Response
                else Invalid credentials
                    Service-->>Middleware: Ok(false)
                    Middleware-->>Client: 401 Unauthorized
                end
            end
        end
    end
```

### Public Endpoints

| Path | Method | Auth Required |
|------|--------|---------------|
| `/v1/ping` | GET | ❌ No |
| `/v1/magic/content` | POST | ✅ Yes |
| `/v1/magic/path` | POST | ✅ Yes |

### Implementation

```rust
pub fn auth_layer(
    auth_service: Arc<dyn AuthenticationService>,
) -> ServiceBuilder<Stack<AuthLayer, Identity>> {
    ServiceBuilder::new()
        .layer(AuthLayer { auth_service })
}

pub struct AuthLayer {
    auth_service: Arc<dyn AuthenticationService>,
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            auth_service: self.auth_service.clone(),
        }
    }
}

pub struct AuthMiddleware<S> {
    inner: S,
    auth_service: Arc<dyn AuthenticationService>,
}

impl<S, B> Service<Request<B>> for AuthMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = /* ... */;
    
    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Check if endpoint is public
        if is_public_endpoint(req.uri().path()) {
            return self.inner.call(req);
        }
        
        // Extract Authorization header
        let auth_header = match req.headers().get("authorization") {
            Some(h) => h,
            None => return ready(Err(AuthError::MissingHeader)),
        };
        
        // Parse Basic auth
        let (username, password) = match extract_basic_auth(auth_header) {
            Ok(creds) => creds,
            Err(e) => return ready(Err(e)),
        };
        
        // Verify credentials
        let auth_service = self.auth_service.clone();
        let fut = self.inner.call(req);
        
        async move {
            let is_valid = auth_service
                .verify_credentials(&username, &password)
                .map_err(|_| AuthError::InternalError)?;
            
            if is_valid {
                Ok(fut.await?)
            } else {
                Err(AuthError::InvalidCredentials)
            }
        }
    }
}

fn extract_basic_auth(header: &HeaderValue) -> Result<(String, String), AuthError> {
    let auth_str = header.to_str().map_err(|_| AuthError::InvalidFormat)?;
    
    if !auth_str.starts_with("Basic ") {
        return Err(AuthError::InvalidFormat);
    }
    
    let encoded = &auth_str[6..];
    let decoded = base64::decode(encoded).map_err(|_| AuthError::InvalidFormat)?;
    let credentials = String::from_utf8(decoded).map_err(|_| AuthError::InvalidFormat)?;
    
    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(AuthError::InvalidFormat);
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

---

## Timeout Middleware

### Class Diagram

```mermaid
classDiagram
    class TimeoutMiddleware {
        +timeout_layer(duration: Duration) Layer
    }
    
    class Duration {
        <<std::time>>
    }
    
    TimeoutMiddleware ..> Duration : uses
    
    note for TimeoutMiddleware "Request timeout enforcement\nDefault: 30 seconds\nPrevents hung requests"
```

### Flow

```mermaid
sequenceDiagram
    participant Client
    participant Middleware as TimeoutMiddleware
    participant Handler
    
    Client->>Middleware: HTTP Request
    Middleware->>Middleware: Start timer (30s)
    Middleware->>Handler: Forward request
    
    alt Handler completes within timeout
        Handler-->>Middleware: Response
        Middleware-->>Client: 200 OK
    else Timeout exceeded
        Middleware->>Middleware: Cancel request
        Middleware-->>Client: 504 Gateway Timeout
    end
```

### Implementation

```rust
use tower::timeout::Timeout;
use std::time::Duration;

pub fn timeout_layer(duration: Duration) -> Timeout {
    Timeout::new(duration)
}

// Usage in router
let app = Router::new()
    .route("/v1/magic/content", post(analyze_content_handler))
    .layer(timeout_layer(Duration::from_secs(30)));
```

### Configuration

| Endpoint | Timeout | Rationale |
|----------|---------|-----------|
| `/v1/ping` | 2s | Health check should be fast |
| `/v1/magic/content` | 30s | File analysis can be slow for large files |
| `/v1/magic/path` | 30s | File I/O + analysis |

---

## Error Handler Middleware

### Class Diagram

```mermaid
classDiagram
    class ErrorHandlerMiddleware {
        +error_handler_layer() Layer
        +handle_error(error: BoxError) Response
        +map_to_http_status(error: &Error) StatusCode
    }
    
    class ErrorResponse {
        +error: ErrorDetail
    }
    
    class ErrorDetail {
        +code: String
        +message: String
        +request_id: RequestId
    }
    
    ErrorHandlerMiddleware ..> ErrorResponse : produces
    ErrorHandlerMiddleware ..> ErrorDetail : uses
    
    note for ErrorHandlerMiddleware "Global error handler\nMaps all errors to HTTP responses\nIncludes request ID in errors"
```

### Flow

```mermaid
flowchart TD
    Error[Error occurs] --> Handler[ErrorHandlerMiddleware]
    Handler --> CheckType{Error type?}
    
    CheckType -->|ApplicationError| MapApp[Map to HTTP status]
    CheckType -->|AuthError| Map401[401 Unauthorized]
    CheckType -->|TimeoutError| Map504[504 Gateway Timeout]
    CheckType -->|Unknown| Map500[500 Internal Server Error]
    
    MapApp --> CreateResponse[Create ErrorResponse]
    Map401 --> CreateResponse
    Map504 --> CreateResponse
    Map500 --> CreateResponse
    
    CreateResponse --> AddRequestId[Add request ID]
    AddRequestId --> Log[Log error]
    Log --> Return[Return JSON response]
    
    style Return fill:#FFB6C1
```

### Implementation

```rust
pub fn error_handler_layer() -> HandleErrorLayer</* ... */> {
    HandleErrorLayer::new(handle_error)
}

async fn handle_error(error: BoxError) -> (StatusCode, Json<ErrorResponse>) {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::GATEWAY_TIMEOUT,
            Json(ErrorResponse::new("timeout", "Request timeout")),
        );
    }
    
    if let Some(auth_error) = error.downcast_ref::<AuthError>() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "authentication_required",
                "Authentication failed",
            )),
        );
    }
    
    // Default: Internal server error
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            "internal_error",
            "Internal server error",
        )),
    )
}
```

## Middleware Composition

```rust
pub fn create_app(/* dependencies */) -> Router {
    Router::new()
        // Routes
        .route("/v1/ping", get(ping_handler))
        .route("/v1/magic/content", post(analyze_content_handler))
        .route("/v1/magic/path", post(analyze_path_handler))
        // Middleware (applied in reverse order)
        .layer(error_handler_layer())          // 4. Handle errors
        .layer(auth_layer(auth_service))       // 3. Authenticate
        .layer(timeout_layer(Duration::from_secs(30)))  // 2. Timeout
        .layer(request_id_layer())             // 1. Request ID (first)
}
```

## Design Rationale

- **Request ID**: Enables distributed tracing and log correlation
- **Timeout**: Prevents hung requests and resource exhaustion
- **Authentication**: Secures endpoints with Basic Auth
- **Error Handler**: Provides consistent error responses
- **Composability**: Tower middleware layers compose cleanly
- **Type Safety**: Axum extractors provide compile-time guarantees
- **Testability**: Each middleware is independently testable
