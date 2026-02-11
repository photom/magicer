# Test Plan: AnalyzeContentUseCase

## test_analyze_content_small_success

**Setup:**
- Mock `MagicRepository` configured to return `MimeType` "text/plain" and description "ASCII text" for a specific buffer.
- `AnalyzeContentUseCase` initialized with the mock repository.
- `AnalyzeContentRequest` with content "Hello World" (below threshold).

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Ok`
- Response contains `MimeType` "text/plain"
- Response contains description "ASCII text"
- `MagicRepository::analyze_buffer` was called exactly once.
- No temporary files were created.

## test_analyze_content_large_success

**Setup:**
- Mock `MagicRepository` configured to return `MimeType` "application/octet-stream" and description "data" for a large buffer.
- `AnalyzeContentUseCase` initialized with the mock repository.
- Configured threshold is 10MB.
- `AnalyzeContentRequest` with 11MB of data.

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Ok`
- Response matches expected MIME and description.
- `MagicRepository::analyze_buffer` was called with the large buffer (or the mmap'ed slice).
- A temporary file was created and subsequently deleted.

## test_analyze_content_empty_rejected

**Setup:**
- `AnalyzeContentUseCase` with any repository.
- `AnalyzeContentRequest` with empty content.

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Err`
- Error variant matches `ApplicationError::ValidationError` or similar.
- `MagicRepository` was never called.

## test_analyze_content_repository_failure

**Setup:**
- Mock `MagicRepository` configured to return `MagicError::AnalysisFailed`.
- `AnalyzeContentUseCase` initialized with the mock repository.

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Err`
- Error wraps the domain `MagicError`.

# Resource Efficiency and Fallback

## test_analyze_content_streaming_constant_memory

**Setup:**
- `AnalyzeContentUseCase` receiving a 100MB stream.
- Monitor heap allocation (using mock stream or instrumented allocator).

**Execution:**
- Stream 100MB to the use case.

**Assertions:**
- Peak memory usage during streaming remains close to the 64KB buffer size.
- Memory does not scale linearly with 100MB.

## test_analyze_content_mmap_fallback_enabled

**Setup:**
- `analysis.mmap_fallback_enabled = true`.
- Mock `MmapHandler` to fail.

**Execution:**
- Analyze a large file (> threshold).

**Assertions:**
- The use case falls back to reading the file into a buffer.
- `MagicRepository::analyze_buffer` is called with the loaded buffer.
- Result is `Ok`.

## test_analyze_content_mmap_fallback_disabled

**Setup:**
- `analysis.mmap_fallback_enabled = false`.
- Mock `MmapHandler` to fail.

**Execution:**
- Analyze a large file.

**Assertions:**
- The use case fails immediately.
- Result is `Err(ApplicationError::Internal)`.

# Test Plan: AnalyzePathUseCase

## test_analyze_path_success

**Setup:**
- Mock `MagicRepository` configured to return result for path "uploads/test.txt".
- `AnalyzePathUseCase` with mock repository and sandbox configured to "uploads/".
- File "uploads/test.txt" exists in the mock/real filesystem.

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Ok`
- Response contains correct analysis.
- `MagicRepository::analyze_file` was called with the correct path.

## test_analyze_path_outside_sandbox_rejected

**Setup:**
- `AnalyzePathUseCase` with sandbox "uploads/".
- `AnalyzePathRequest` with path "/etc/passwd".

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Err`
- Error indicates boundary violation.

## test_analyze_path_not_found

**Setup:**
- `AnalyzePathUseCase`.
- `AnalyzePathRequest` with path "uploads/missing.txt" (does not exist).

**Execution:**
- Call `use_case.execute(request)`

**Assertions:**
- Result is `Err`
- Error variant is `ApplicationError::NotFound`.

# Test Plan: HealthCheckUseCase

## test_health_check_success

**Setup:**
- `HealthCheckUseCase`.

**Execution:**
- Call `use_case.execute()`

**Assertions:**
- Returns `Ok` with status "up".
