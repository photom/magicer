# Test Plan: AnalyzeContentUseCase

## test_analyze_in_memory_success

**Setup:**
- `AnalyzeContentUseCase`.
- Small data stream.

**Execution:**
- Call `use_case.analyze_in_memory(request_id, filename, stream)`

**Assertions:**
- Result is `Ok`
- `MagicRepository::analyze_buffer` called.

## test_analyze_to_temp_file_success

**Setup:**
- `AnalyzeContentUseCase`.
- Data stream.

**Execution:**
- Call `use_case.analyze_to_temp_file(request_id, filename, stream)`

**Assertions:**
- Result is `Ok`
- `temp_storage.create_temp_file()` called.
- `MmapHandler` used.
- `MagicRepository::analyze_buffer` called with mmap slice.

## test_analyze_in_memory_with_empty_content_returns_error

**Setup:**
- `AnalyzeContentUseCase`.
- Empty stream.

**Execution:**
- Call `use_case.analyze_in_memory(...)`

**Assertions:**
- Result is `Err(ApplicationError::BadRequest)`

## test_analyze_in_memory_timeout_returns_error

**Setup:**
- Mock `MagicRepository` that hangs or takes longer than `analysis_timeout_secs`.
- Configured timeout (e.g., 1s).

**Execution:**
- Call `use_case.analyze_in_memory(...)`

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
- File opened and mapped via `MmapHandler`.
- `MagicRepository::analyze_buffer` called with mmap slice.

## test_execute_with_missing_file_returns_error

**Setup:**
- `AnalyzePathUseCase`.
- Missing file path.

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
