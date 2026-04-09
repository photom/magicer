# Test Plan: FFI and Safe Wrappers

# Raw FFI Declarations (`ffi.rs`)

## test_ffi_linkage

**Setup:**
- None (Linkage test).

**Execution:**
- Call a simple FFI function like `magic_version()` (if available) or check if `magic_open` pointer is reachable.

**Assertions:**
- The binary links correctly with `libmagic.so`.
- No "undefined symbol" errors at runtime.

# Safe Wrapper (`wrapper.rs`)

## test_magic_cookie_lifecycle

**Setup:**
- `MagicCookie::new(MagicFlags::MIME_TYPE)`.

**Execution:**
- Create cookie.
- Load default database.
- Drop cookie.

**Assertions:**
- Cookie creation returns `Ok`.
- Database load returns `Ok`.
- No memory leaks (verified via Valgrind in CI if possible).
- `magic_close` is called on drop.

## test_magic_cookie_send_sync_boundary

**Setup:**
- `MagicCookie`.

**Execution:**
- Attempt to move `MagicCookie` to another thread.
- Attempt to share `&MagicCookie` between threads.

**Assertions:**
- Compilation fails (verified via `compiletest_rs` or similar, or just manual check of `!Send` and `!Sync` markers).

## test_magic_cookie_error_handling

**Setup:**
- `MagicCookie` initialized.
- Attempt to load a non-existent database file.

**Execution:**
- Call `cookie.load("non_existent.mgc")`.

**Assertions:**
- Returns `Err(MagicError::DatabaseLoad)`.
- Error message contains descriptive string from `magic_error()`.

## test_magic_cookie_analyze_buffer

**Setup:**
- `MagicCookie` with `MIME_TYPE` flag.
- A buffer containing PNG magic bytes: `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]`.

**Execution:**
- Call `cookie.analyze_buffer(buffer)`.

**Assertions:**
- Returns `Ok("image/png")`.

## test_magic_cookie_analyze_buffer_invalid_utf8

**Setup:**
- Mock or force libmagic to return a non-UTF8 string (might be hard with real libmagic, but can test the conversion logic).

**Execution:**
- Call analysis.

**Assertions:**
- Returns `Err(MagicError::InvalidUtf8)` or handles it gracefully by returning a lossy string if that's the policy (check `LIBMAGIC_FFI.md`). *Correction: `LIBMAGIC_FFI.md` says "Convert C string to Rust String (copy)" and "Check encoding".*

# Thread-Safe Repository (`lib.rs`)

## test_repository_concurrent_access

**Setup:**
- `LibmagicRepository` wrapping `Arc<Mutex<MagicCookie>>`.
- 10 concurrent threads calling `analyze_buffer`.

**Execution:**
- Run 10 threads in parallel.

**Assertions:**
- All threads return correct results.
- No race conditions or crashes.
- Mutex correctly serializes access.

# Miri / Memory Safety

## test_ffi_miri_clean

**Setup:**
- Running tests under Miri.

**Execution:**
- Run the unit test suite for `wrapper.rs`.

**Assertions:**
- Miri reports zero undefined behavior, memory leaks, or pointer misuses.
