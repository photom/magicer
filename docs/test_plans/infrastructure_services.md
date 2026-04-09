# Test Plan: LibmagicRepository (Integration)

## test_libmagic_analyze_text_buffer

**Setup:**
- `LibmagicRepository` initialized with default magic database.
- A buffer containing "Hello, world!
".

**Execution:**
- Call `repository.analyze_buffer(buffer, None)`

**Assertions:**
- Result is `Ok`
- MIME type is "text/plain"
- Description contains "ASCII text"

# Test Plan: PathSandbox

## test_sandbox_within_boundary

**Setup:**
- `PathSandbox` with base directory `/tmp/magicer/sandbox`.
- Ensure directory exists.

**Execution:**
- Call `sandbox.validate_path("test.txt")`

**Assertions:**
- Returns `Ok(canonical_path)`
- `canonical_path` starts with `/tmp/magicer/sandbox`

## test_sandbox_escape_attempt_rejected

**Setup:**
- `PathSandbox` with base directory `/tmp/magicer/sandbox`.

**Execution:**
- Call `sandbox.validate_path("../../etc/passwd")`

**Assertions:**
- Returns `Err`
- Error indicates path traversal or boundary violation.

# Test Plan: TempFileHandler

## test_temp_file_creation_and_cleanup

**Setup:**
- `TempFileHandler` with a specific temp directory.

**Execution:**
- Call `handler.create_temp_file()`
- Write some data.
- Note the file path.
- Drop the handler.

**Assertions:**
- File was created at the path.
- Data was written correctly.
- After drop, the file no longer exists.

## test_temp_file_collision_retry

**Setup:**
- `TempFileHandler`.
- Manually create a file with the name that the handler is about to generate (may require mocking the name generator if possible, or high-concurrency test).

**Execution:**
- Call `handler.create_temp_file()`

**Assertions:**
- Handler successfully creates a file with a DIFFERENT name.
- No error is returned despite the initial collision.

# Test Plan: MmapHandler

## test_mmap_readonly_enforced

**Setup:**
- `MmapHandler` mapping a temporary file.

**Execution:**
- Create mapping.
- Attempt to write to the mapped memory (using unsafe pointer cast to `*mut u8` if necessary for testing).

**Assertions:**
- Operation triggers a segmentation fault or the system correctly enforces read-only if checked via `mprotect` etc. (In Rust, we just ensure we only expose `&[u8]`).

## test_mmap_sigbus_handling

**Setup:**
- `MmapHandler` mapping a file.
- Another process or thread truncates the file while it's mapped.

**Execution:**
- Access the mapped memory at the original size.

**Assertions:**
- The signal handler catches SIGBUS.
- The system returns a clean `MagicError::IoError` or similar instead of crashing the entire process.
