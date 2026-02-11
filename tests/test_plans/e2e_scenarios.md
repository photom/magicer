# Test Plan: E2E Scenarios

## test_ping_endpoint

**Execution:**
- GET `/v1/ping`

**Assertions:**
- Status 200 OK.

## test_content_analysis_success

**Setup:**
- Valid Basic Auth.
- Binary data (PDF).

**Execution:**
- POST `/v1/magic/content?filename=test.pdf`

**Assertions:**
- Status 200 OK.
- JSON body matches `MagicResponse`.
- MIME type is `application/pdf`.

## test_path_analysis_success

**Setup:**
- Valid Basic Auth.
- File `test.png` exists in sandbox directory.

**Execution:**
- POST `/v1/magic/path?filename=test.png&path=test.png`

**Assertions:**
- Status 200 OK.
- MIME type is `image/png`.

## test_auth_required

**Execution:**
- POST `/v1/magic/content` without Authorization header.

**Assertions:**
- Status 401 Unauthorized.
