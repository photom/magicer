# Test Plan: Presentation Layer (Axum)

# Middleware and Layer Tests

## test_request_id_middleware

**Assertions:**
- Response contains `X-Request-ID` header.
- Value is valid UUID v4.

## test_auth_middleware_success

**Setup:**
- Valid `Authorization: Basic ...` header.

**Assertions:**
- Status `200 OK`.

## test_body_size_limit_enforced

**Setup:**
- `server.limits.max_body_size_mb = 1`.
- Upload 2MB of data.

**Assertions:**
- Status `413 Payload Too Large`.

## test_global_request_timeout

**Setup:**
- `server.timeouts.read_timeout_secs = 1`.
- Handler that takes 2s.

**Assertions:**
- Status `408 Request Timeout`.

# Handler Logic (Streaming)

## test_analyze_content_switches_to_file_at_threshold

**Setup:**
- `analysis.large_file_threshold_mb = 1`.
- Upload 2MB stream.

**Execution:**
- POST `/v1/magic/content`.

**Assertions:**
- Status `200 OK`.
- Internal trace shows switching to file-based handling.
- `execute_from_file` use case method called.

## test_analyze_content_memory_only_below_threshold

**Setup:**
- `analysis.large_file_threshold_mb = 10`.
- Upload 1KB data.

**Assertions:**
- Internal trace shows processing in memory.
- `execute` use case method called.
