# Test Plan: SOLID Refactoring of AnalyzeContentUseCase

## Background
The previous `analyze_stream` method was a monolithic method that handled both in-memory buffering and file-based streaming using internal flags and threshold logic. This violated the Single Responsibility Principle (SRP) and made the code harder to maintain and test.

The refactoring split this logic into two distinct, focused methods:
- `analyze_in_memory`: For small payloads.
- `analyze_to_temp_file`: For chunked or large payloads.

The decision logic (dispatching) was moved to the Presentation Layer (HTTP Handler).

## test_analyze_in_memory_success
**Purpose:** Verify that `analyze_in_memory` correctly collects a stream into a buffer and performs analysis.
**Setup:**
- `AnalyzeContentUseCase` with mock repository.
- Small data stream (e.g., 1KB).
**Execution:**
- Call `use_case.analyze_in_memory(request_id, filename, stream)`
**Assertions:**
- Result is `Ok`.
- `MagicRepository::analyze_buffer` is called with the full collected data.

## test_analyze_to_temp_file_success
**Purpose:** Verify that `analyze_to_temp_file` correctly streams data to a temporary file and performs analysis via mmap.
**Setup:**
- `AnalyzeContentUseCase` with mock repository and fake temp storage.
- Data stream.
**Execution:**
- Call `use_case.analyze_to_temp_file(request_id, filename, stream)`
**Assertions:**
- Result is `Ok`.
- `temp_storage.create_temp_file()` is called.
- `MagicRepository::analyze_file` is called with the temp file path.

## test_handler_dispatch_to_in_memory
**Purpose:** Verify the handler correctly chooses in-memory analysis for small non-chunked requests.
**Setup:**
- `Content-Length` header below threshold.
- No `Transfer-Encoding: chunked` header.
**Execution:**
- POST `/v1/magic/content`
**Assertions:**
- Use case's `analyze_in_memory` is called (verify via mock or side-effect).

## test_handler_dispatch_to_temp_file_via_chunked
**Purpose:** Verify the handler correctly chooses file-based analysis for chunked requests.
**Setup:**
- `Transfer-Encoding: chunked` header.
**Execution:**
- POST `/v1/magic/content`
**Assertions:**
- Use case's `analyze_to_temp_file` is called.

## test_handler_dispatch_to_temp_file_via_threshold
**Purpose:** Verify the handler correctly chooses file-based analysis for large non-chunked requests.
**Setup:**
- `Content-Length` header above threshold.
**Execution:**
- POST `/v1/magic/content`
**Assertions:**
- Use case's `analyze_to_temp_file` is called.
