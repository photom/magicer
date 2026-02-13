# Test Plan: AnalyzeContentUseCase

## test_execute_with_small_content_returns_success

**Setup:**
- Mock `MagicRepository` returning `MimeType` "text/plain".
- `AnalyzeContentUseCase` with default config.
- Small buffer data.

**Execution:**
- Call `use_case.execute(request_id, filename, data)`

**Assertions:**
- Result is `Ok`
- `MagicRepository::analyze_buffer` called with correct data.

## test_execute_from_file_returns_success

**Setup:**
- `AnalyzeContentUseCase`.
- Valid file on disk.

**Execution:**
- Call `use_case.execute_from_file(request_id, filename, path)`

**Assertions:**
- Result is `Ok`
- File is mapped using `MmapHandler`.
- `MagicRepository::analyze_buffer` called with mmap slice.

## test_execute_with_empty_content_returns_error

**Setup:**
- `AnalyzeContentUseCase`.
- Empty byte slice.

**Execution:**
- Call `use_case.execute(...)`

**Assertions:**
- Result is `Err(ApplicationError::BadRequest)`

## test_execute_timeout_returns_error

**Setup:**
- Mock `MagicRepository` that hangs or takes longer than `analysis_timeout_secs`.
- Configured timeout (e.g., 1s).

**Execution:**
- Call `use_case.execute(...)`

**Assertions:**
- Result is `Err`
- Error message contains "Analysis timed out".

# Test Plan: AnalyzePathUseCase

## test_execute_with_valid_path_returns_success

**Setup:**
- `AnalyzePathUseCase` with mock repository and sandbox.
- File exists in sandbox.

**Execution:**
- Call `use_case.execute(...)`

**Assertions:**
- Result is `Ok`
- `SandboxService::resolve_path` called.
- `MagicRepository::analyze_file` called with resolved path.

## test_execute_with_missing_file_returns_error

**Setup:**
- `AnalyzePathUseCase`.
- Mock repository returning `MagicError::FileNotFound`.

**Execution:**
- Call `use_case.execute(...)`

**Assertions:**
- Result is `Err(ApplicationError::NotFound)`

## test_execute_with_timeout_returns_error

**Setup:**
- Mock repository that delays longer than timeout.

**Execution:**
- Call `use_case.execute(...)`

**Assertions:**
- Result is `Err`
- Error message contains "Analysis timed out".
