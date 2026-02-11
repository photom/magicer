# Test Plan: Presentation Layer (Axum)

# Middleware Tests

## test_request_id_middleware

**Setup:**
- Axum router with `RequestIdMiddleware`.
- Mock handler that returns 200 OK.

**Execution:**
- Send any HTTP request.

**Assertions:**
- Response contains `X-Request-ID` header.
- The header value is a valid UUID v4.
- The same ID is available in request extensions during handling.

## test_auth_middleware_success

**Setup:**
- Axum router with `AuthMiddleware` (configured with username "admin", password "secret").
- Request with header `Authorization: Basic YWRtaW46c2VjcmV0`.

**Execution:**
- Send the request to a protected route.

**Assertions:**
- Status code is 200 OK.

## test_auth_middleware_failure

**Setup:**
- Axum router with `AuthMiddleware`.
- Request with invalid credentials or missing header.

**Execution:**
- Send the request to a protected route.

**Assertions:**
- Status code is 401 Unauthorized.
- Response body matches the error JSON format.

## test_timeout_middleware

**Setup:**
- Axum router with `TimeoutMiddleware` set to 1s.
- Handler that sleeps for 2s.

**Execution:**
- Send request to the slow handler.

**Assertions:**
- Status code is 408 Request Timeout or 504 Gateway Timeout.

# Handler Tests (Integration)

## test_post_content_success

**Setup:**
- Test server running with `AppState`.
- Valid content in request body.
- Valid Basic Auth header.

**Execution:**
- POST `/v1/magic/content?filename=test.txt`

**Assertions:**
- Status code is 200 OK.
- Response body is valid JSON matching `MagicResponse`.
- Contains `mime_type` and `description`.

## test_post_path_success

**Setup:**
- Test server.
- Valid file path in request body (JSON).
- File exists in sandbox.
- Valid Basic Auth header.

**Execution:**
- POST `/v1/magic/path` with `{"path": "uploads/test.txt"}`

**Assertions:**
- Status code is 200 OK.
- Response body contains analysis results.

## test_error_response_format

**Setup:**
- Test server.
- Trigger a validation error (e.g., empty filename).

**Execution:**
- POST `/v1/magic/content?filename=`

**Assertions:**
- Status code is 400 Bad Request.
- Response body: `{"error": "...", "request_id": "..."}`
