# Test Plan: Operational and System Integrity

# Graceful Shutdown

## test_graceful_shutdown_drains_requests

**Setup:**
- Start server.
- Send a request that takes 5 seconds to process (e.g., large file or mock delay).
- While request is in-flight, send `SIGTERM` to the process.

**Execution:**
- Wait for process to exit.

**Assertions:**
- The in-flight request completes successfully with 200 OK.
- The process exits within the 10-second window.
- New requests sent *after* `SIGTERM` are rejected or the port is closed.

# Resource Limits

## test_fd_limit_rejection

**Setup:**
- Configure `server.max_open_files` to a low value (e.g., 20).
- Open 20+ concurrent connections/files.

**Execution:**
- Attempt to open another connection.

**Assertions:**
- Server returns `503 Service Unavailable`.
- Error message contains "Too many open files".
- Metrics `fd_limit_rejections_total` increments.

## test_disk_space_preflight_rejection

**Setup:**
- Configure `analysis.min_free_space_mb` to 1000 (1GB).
- Mock the filesystem to report 500MB free.

**Execution:**
- POST a large content request.

**Assertions:**
- Server returns `507 Insufficient Storage` *before* starting the upload stream.
- Error message specifies insufficient space.

# Background Tasks

## test_orphaned_file_cleanup

**Setup:**
- Manually place a file in `analysis.temp_dir` with a modification time 2 hours ago.
- Start server with `analysis.temp_file_max_age_secs = 3600` (1 hour).

**Execution:**
- Wait for background cleanup task (configured for every 5 mins, maybe speed up for test).

**Assertions:**
- The orphaned file is deleted.
- Active temp files (younger than 1 hour) are NOT deleted.

# Startup Validation

## test_startup_invalid_config_fails

**Setup:**
- Provide `config.toml` with `server.port = 99999` (invalid).

**Execution:**
- Attempt to start the server.

**Assertions:**
- Process exits with non-zero code.
- Error message indicates port range validation failure.

## test_startup_sandbox_missing_fails

**Setup:**
- Configure `sandbox.base_dir` to a path that doesn't exist.

**Execution:**
- Start server.

**Assertions:**
- Process exits with panic/error "Failed to validate configuration".
- Error cause indicates "File or directory not found".
