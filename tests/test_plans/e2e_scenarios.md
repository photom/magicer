# Test Plan: Full Workflow (E2E)

This test plan describes the end-to-end verification of the Magicer API, ensuring all layers (Presentation, Application, Domain, and Infrastructure) integrate correctly to fulfill the API contract.

## 1. Health Check Workflow

### test_ping_endpoint_success
**Setup:**
- Server is running with default configuration.

**Execution:**
- GET `/v1/ping`

**Assertions:**
- Status `200 OK`.
- Content-Type is `application/json`.
- Body contains `message: "pong"`.
- Body contains a valid `request_id` (UUID format).

---

## 2. Content Analysis Workflows

### test_analyze_content_pdf_success
**Setup:**
- Basic Authentication enabled (admin:secret).
- Valid PDF header data (`%PDF-1.4`).

**Execution:**
- POST `/v1/magic/content?filename=test.pdf`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`
- Body: Raw binary PDF bytes.

**Assertions:**
- Status `200 OK`.
- Body matches `MagicResult` schema:
    - `request_id` is present.
    - `filename` is "test.pdf".
    - `result.mime_type` is "application/pdf".
    - `result.description` contains "PDF document".
- `X-Request-ID` header is present in the response and matches the body.

### test_analyze_content_large_file_success
**Setup:**
- Server configured with a small large-file threshold (e.g., 1MB for testing).
- 2MB of valid PNG data.

**Execution:**
- POST `/v1/magic/content?filename=large.png`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`
- Body: 2MB PNG data.

**Assertions:**
- Status `200 OK`.
- `result.mime_type` is "image/png".
- System correctly handled streaming to temp file and mmap analysis (verified via logs or side-effects if possible).

---

## 3. Path Analysis Workflows

### test_analyze_path_png_success
**Setup:**
- File `test.png` pre-positioned in the configured sandbox directory.
- File contains valid PNG signature.

**Execution:**
- POST `/v1/magic/path?filename=test.png&path=test.png`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`

**Assertions:**
- Status `200 OK`.
- `result.mime_type` is "image/png".
- `result.description` contains "PNG image data".

### test_analyze_path_not_found
**Setup:**
- Path `missing.bin` does not exist in sandbox.

**Execution:**
- POST `/v1/magic/path?filename=missing.bin&path=missing.bin`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`

**Assertions:**
- Status `404 Not Found`.
- Body contains standardized `ErrorResponse` with `error` and `request_id`.

---

## 4. Security and Validation Workflows

### test_auth_required_rejection
**Execution:**
- POST `/v1/magic/content?filename=any.txt`
- Body: "some text"
- **No Authorization header.**

**Assertions:**
- Status `401 Unauthorized`.
- Body contains standardized `ErrorResponse`.

### test_invalid_filename_rejection
**Execution:**
- POST `/v1/magic/content?filename=bad/name.txt`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`

**Assertions:**
- Status `400 Bad Request`.
- Body contains `error` message specifying invalid filename.

### test_path_traversal_rejection
**Execution:**
- POST `/v1/magic/path?filename=etc&path=../../etc/passwd`
- Header: `Authorization: Basic YWRtaW46c2VjcmV0`

**Assertions:**
- Status `400 Bad Request` or `403 Forbidden` (depending on validation layer hit first).
- Access to external files is strictly blocked.
