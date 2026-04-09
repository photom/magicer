# Test Plan: Operational and System Integrity

# Directory Management

## test_validate_creates_missing_directories

**Setup:**
- `ServerConfig` with non-existent paths for `sandbox.base_dir` and `analysis.temp_dir`.

**Execution:**
- Call `config.validate()`.

**Assertions:**
- Result is `Ok`.
- The missing directories now exist on the filesystem.

# CLI Argument Parsing

## test_cli_config_path_override

**Setup:**
- Custom config file exists at `/tmp/custom.toml`.

**Execution:**
- Run binary with `--config /tmp/custom.toml`.

**Assertions:**
- Application loads configuration from the specified file.

# Graceful Shutdown

## test_graceful_shutdown_drains_requests

**Setup:**
- Start server.
- Send a request that takes 5 seconds to process.
- While request is in-flight, send `SIGTERM`.

**Execution:**
- Wait for process to exit.

**Assertions:**
- The in-flight request completes successfully.
- Process exits within the shutdown window.

# Resource Limits

## test_fd_limit_applied_at_startup

**Setup:**
- `server.max_open_files = 1024`.

**Execution:**
- Start application.

**Assertions:**
- Process `NOFILE` limit is set to 1024 (verified via `/proc/self/limits` in integration test).

## test_concurrency_limit_applied

**Setup:**
- `server.max_connections = 2`.

**Execution:**
- Send 3 concurrent requests.

**Assertions:**
- 2 requests proceed.
- 1 request is either queued or rejected depending on `tower` configuration.

# Background Tasks

## test_orphaned_file_cleanup

**Setup:**
- Manually place a file in `analysis.temp_dir` with a modification time older than `temp_file_max_age_secs`.

**Execution:**
- Wait for background cleanup task to run.

**Assertions:**
- The orphaned file is deleted.
- Newer files are preserved.

# Resource Limits

## test_disk_space_preflight_rejection

**Setup:**
- Configure `analysis.min_free_space_mb` to a value higher than available disk space.

**Execution:**
- POST a content analysis request that triggers file-based handling.

**Assertions:**
- Server returns `507 Insufficient Storage` before processing the body.
- Error message specifies insufficient disk space.

# Analysis Robustness

## test_mmap_fallback_to_buffer

**Setup:**
- `analysis.mmap_fallback_enabled = true`.
- Trigger a scenario where `mmap` fails (e.g. mock failure).

**Execution:**
- Perform content analysis.

**Assertions:**
- Analysis succeeds by falling back to traditional buffer-based reading.
