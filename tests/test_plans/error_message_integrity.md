# Test Plan: Error Message Integrity

# Error Formatting Standards

## test_error_message_structure_compliance

**Setup:**
- Trigger various errors (Validation, Auth, I/O, Analysis).

**Execution:**
- Inspect the `error` field in JSON responses.

**Assertions:**
- Every message follows the `"Failed to {operation}: {cause}"` pattern.
- Example: `"Failed to create temp file: disk full"`.
- No generic messages like `"Operation failed"` or `"Error"`.

# Context Preservation

## test_io_error_includes_details

**Setup:**
- Force an I/O error during streaming (e.g., mock disk full mid-stream).

**Execution:**
- Capture the 507/500 response.

**Assertions:**
- Error message includes the offset or specific operation: `"Failed to write chunk at offset 10485760: No space left on device"`.

## test_domain_error_propagation

**Setup:**
- Trigger a `ValidationError` from the domain layer.

**Execution:**
- Capture response.

**Assertions:**
- The presentation layer does not swallow the specific cause.
- Resulting JSON: `{"error": "Failed to validate filename: exceeds maximum length", ...}` (exact wording depends on domain impl).

# Sanitization

## test_internal_error_sanitization

**Setup:**
- Trigger a 500 Internal Server Error (e.g., libmagic crash/null).

**Execution:**
- Capture response.

**Assertions:**
- The external JSON response contains a human-readable summary.
- The internal logs (not the HTTP response) contain the full technical backtrace/FFI details.
- `request_id` is present in both for correlation.
