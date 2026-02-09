# Configuration Reference <!-- omit in toc -->

Complete reference for all configuration options in `config.toml`.

- [File Location](#file-location)
- [Configuration Loading Priority](#configuration-loading-priority)
- [Server Configuration](#server-configuration)
  - [`[server]`](#server)
    - [`server.host`](#serverhost)
    - [`server.port`](#serverport)
    - [`server.max_connections`](#servermax_connections)
    - [`server.backlog`](#serverbacklog)
- [Timeout Configuration](#timeout-configuration)
  - [`[server.timeouts]`](#servertimeouts)
    - [`server.timeouts.read_timeout_secs`](#servertimeoutsread_timeout_secs)
    - [`server.timeouts.write_timeout_secs`](#servertimeoutswrite_timeout_secs)
    - [`server.timeouts.analysis_timeout_secs`](#servertimeoutsanalysis_timeout_secs)
    - [`server.timeouts.keepalive_secs`](#servertimeoutskeepalive_secs)
- [Request Limits](#request-limits)
  - [`[server.limits]`](#serverlimits)
    - [`server.limits.max_body_size_mb`](#serverlimitsmax_body_size_mb)
    - [`server.limits.max_uri_length`](#serverlimitsmax_uri_length)
    - [`server.limits.max_header_size`](#serverlimitsmax_header_size)
- [Sandbox Configuration](#sandbox-configuration)
  - [`[sandbox]`](#sandbox)
    - [`sandbox.base_dir`](#sandboxbase_dir)
- [Authentication Configuration](#authentication-configuration)
  - [`[auth]`](#auth)
    - [`auth.username`](#authusername)
    - [`auth.password`](#authpassword)
- [Magic Database Configuration](#magic-database-configuration)
  - [`[magic]`](#magic)
    - [`magic.database_path`](#magicdatabase_path)
- [Logging Configuration](#logging-configuration)
  - [`[logging]`](#logging)
    - [`logging.level`](#logginglevel)
    - [`logging.format`](#loggingformat)
- [Complete Configuration Example](#complete-configuration-example)
  - [Minimal Configuration](#minimal-configuration)
  - [Full Configuration with All Options](#full-configuration-with-all-options)
  - [Development Configuration](#development-configuration)
  - [Production Configuration](#production-configuration)
- [Environment Variable Reference](#environment-variable-reference)
- [Configuration Validation](#configuration-validation)
- [Security Best Practices](#security-best-practices)
- [Troubleshooting](#troubleshooting)
  - [Configuration File Not Found](#configuration-file-not-found)
  - [Invalid Configuration Value](#invalid-configuration-value)
  - [Sandbox Directory Issues](#sandbox-directory-issues)
  - [Port Already in Use](#port-already-in-use)
- [Related Documentation](#related-documentation)


## File Location

| Environment | Default Path | Override |
|-------------|-------------|----------|
| Development | `./config/config.toml` | `MAGICER_CONFIG_PATH` |
| Production | `/etc/magicer/config.toml` | `MAGICER_CONFIG_PATH` |

## Configuration Loading Priority

1. **Environment Variables** (highest priority)
2. **TOML Configuration File**
3. **Default Values** (lowest priority)

---

## Server Configuration

### `[server]`

Main server settings for network binding and connection management.

#### `server.host`

**Type:** String  
**Default:** `"0.0.0.0"`  
**Environment:** `MAGICER_HOST`  
**Description:** IP address to bind the HTTP server.

**Valid Values:**
- `"0.0.0.0"` - Bind to all network interfaces
- `"127.0.0.1"` - Localhost only
- Any valid IPv4 address

**Example:**
```toml
[server]
host = "0.0.0.0"
```

#### `server.port`

**Type:** Unsigned integer (u16)  
**Default:** `8080`  
**Environment:** `MAGICER_PORT`  
**Description:** TCP port number for HTTP server.

**Valid Range:** 1 - 65535  
**Recommended:** 8080 (development), 80/443 (production with reverse proxy)

**Example:**
```toml
[server]
port = 8080
```

#### `server.max_connections`

**Type:** Unsigned integer (usize)  
**Default:** `1000`  
**Description:** Maximum number of concurrent active TCP connections.

**Valid Range:** 1 - 10000  
**Note:** Connections exceeding this limit enter the backlog queue.

**Example:**
```toml
[server]
max_connections = 1000
```

#### `server.backlog`

**Type:** Unsigned integer (u32)  
**Default:** `1024`  
**Description:** OS-level TCP listen queue size for pending connections.

**Valid Range:** 1 - 65535  
**Note:** Connections exceeding backlog are refused by the OS.

**Example:**
```toml
[server]
backlog = 1024
```

---

## Timeout Configuration

### `[server.timeouts]`

Timeout values controlling request/response lifecycle.

#### `server.timeouts.read_timeout_secs`

**Type:** Unsigned integer (u64)  
**Default:** `60`  
**Unit:** Seconds  
**Description:** Maximum time to receive complete HTTP request (headers + body).

**Valid Range:** 1 - 600  
**Purpose:** Prevent slow-read DoS attacks.

**Example:**
```toml
[server.timeouts]
read_timeout_secs = 60
```

#### `server.timeouts.write_timeout_secs`

**Type:** Unsigned integer (u64)  
**Default:** `60`  
**Unit:** Seconds  
**Description:** Maximum time to send complete HTTP response to client.

**Valid Range:** 1 - 600  
**Purpose:** Prevent slow-send attacks and resource exhaustion.

**Example:**
```toml
[server.timeouts]
write_timeout_secs = 60
```

#### `server.timeouts.analysis_timeout_secs`

**Type:** Unsigned integer (u64)  
**Default:** `30`  
**Unit:** Seconds  
**Description:** Maximum time for libmagic file analysis operation.

**Valid Range:** 1 - 300  
**Purpose:** Prevent indefinite blocking on complex files.

**Example:**
```toml
[server.timeouts]
analysis_timeout_secs = 30
```

#### `server.timeouts.keepalive_secs`

**Type:** Unsigned integer (u64)  
**Default:** `75`  
**Unit:** Seconds  
**Description:** HTTP keep-alive timeout for idle connections.

**Valid Range:** 1 - 600  
**Purpose:** Balance connection reuse with resource cleanup.

**Example:**
```toml
[server.timeouts]
keepalive_secs = 75
```

---

## Request Limits

### `[server.limits]`

Size constraints for HTTP requests.

#### `server.limits.max_body_size_mb`

**Type:** Unsigned integer (u64)  
**Default:** `100`  
**Unit:** Megabytes  
**Description:** Maximum HTTP request body size.

**Valid Range:** 1 - 1024  
**Purpose:** Prevent memory exhaustion from large payloads.

**Example:**
```toml
[server.limits]
max_body_size_mb = 100
```

#### `server.limits.max_uri_length`

**Type:** Unsigned integer (usize)  
**Default:** `8192`  
**Unit:** Bytes  
**Description:** Maximum URI length including query parameters.

**Valid Range:** 256 - 65536  
**Purpose:** Prevent buffer overflow attacks.

**Example:**
```toml
[server.limits]
max_uri_length = 8192
```

#### `server.limits.max_header_size`

**Type:** Unsigned integer (usize)  
**Default:** `16384`  
**Unit:** Bytes  
**Description:** Maximum total size of all HTTP request headers.

**Valid Range:** 1024 - 65536  
**Purpose:** Prevent header-based DoS attacks.

**Example:**
```toml
[server.limits]
max_header_size = 16384
```

---

## Sandbox Configuration

### `[sandbox]`

File system access restrictions for path-based analysis.

#### `sandbox.base_dir`

**Type:** String (absolute path)  
**Default:** `/var/lib/magicer/files`  
**Environment:** `MAGICER_SANDBOX_DIR`  
**Description:** Root directory for all file path operations.

**Requirements:**
- Must be an absolute path
- Directory must exist and be readable
- Server process must have appropriate permissions

**Security:** All relative paths in API requests are resolved within this directory.

**Example:**
```toml
[sandbox]
base_dir = "/var/lib/magicer/files"
```

---

## Authentication Configuration

### `[auth]`

HTTP Basic Authentication settings.

#### `auth.username`

**Type:** String  
**Default:** None (required)  
**Environment:** `MAGICER_AUTH_USERNAME`  
**Description:** Username for HTTP Basic Authentication.

**Requirements:**
- Non-empty string
- Must match client-provided username

**Example:**
```toml
[auth]
username = "api_user"
```

#### `auth.password`

**Type:** String  
**Default:** None (required)  
**Environment:** `MAGICER_AUTH_PASSWORD`  
**Description:** Password for HTTP Basic Authentication.

**Requirements:**
- Non-empty string
- Stored in plain text (use environment variable in production)
- Compared using constant-time algorithm

**Security Recommendation:** Use `MAGICER_AUTH_PASSWORD` environment variable instead of storing in TOML file.

**Example:**
```toml
[auth]
password = "secret_password"
```

---

## Analysis Configuration

### `[analysis]`

File analysis behavior settings.

#### `analysis.large_file_threshold_mb`

**Type:** Unsigned integer (u64)  
**Default:** `10`  
**Unit:** Megabytes  
**Description:** Size threshold above which content is streamed to temporary file instead of analyzed in memory.

**Valid Range:** 1 - 100  
**Purpose:** Reduce memory consumption for large file analysis.

**Trade-offs:**
- **Lower values** - Less memory usage, more disk I/O
- **Higher values** - More memory usage, less disk I/O

**Example:**
```toml
[analysis]
large_file_threshold_mb = 10
```

#### `analysis.write_buffer_size_kb`

**Type:** Unsigned integer (u64)  
**Default:** `64`  
**Unit:** Kilobytes  
**Description:** Buffer size for streaming content to temporary files.

**Valid Range:** 4 - 1024  
**Purpose:** Control memory vs I/O performance trade-off when writing temporary files.

**Recommended:**
- Fast disks (SSD): 64-128 KB
- Slow disks (HDD): 256-512 KB
- Network storage: 128-256 KB

**Example:**
```toml
[analysis]
write_buffer_size_kb = 64
```

#### `analysis.temp_dir`

**Type:** String (absolute path)  
**Default:** `/tmp/magicer`  
**Description:** Directory for temporary files during large content analysis.

**Requirements:**
- Must be an absolute path
- Directory must exist or be creatable
- Server process must have write permissions
- Should be on a filesystem with sufficient space

**Recommendations:**
- Use tmpfs for performance (if sufficient RAM)
- Ensure at least 10GB free space for production
- Separate from main data directories

**Example:**
```toml
[analysis]
temp_dir = "/tmp/magicer"
```

---

## Magic Database Configuration

### `[magic]`

libmagic library settings.

#### `magic.database_path`

**Type:** String (optional, absolute path)  
**Default:** System default (`/usr/share/misc/magic.mgc`)  
**Description:** Path to custom magic database file.

**When to Use:**
- Custom file type detection rules
- Alternative magic database
- Testing with modified rules

**If Not Set:** Uses system-provided magic database.

**Example:**
```toml
[magic]
database_path = "/usr/local/share/magic/custom.mgc"
```

---

## Logging Configuration

### `[logging]`

Logging behavior and output format.

#### `logging.level`

**Type:** String  
**Default:** `"info"`  
**Environment:** `RUST_LOG`  
**Description:** Logging verbosity level.

**Valid Values:**
- `"error"` - Errors only
- `"warn"` - Warnings and errors
- `"info"` - Informational messages (default)
- `"debug"` - Debug information
- `"trace"` - Verbose trace logging

**Example:**
```toml
[logging]
level = "info"
```

**Advanced Filtering (via RUST_LOG):**
```bash
export RUST_LOG="magicer=debug,tower_http=info,hyper=warn"
```

#### `logging.format`

**Type:** String  
**Default:** `"json"`  
**Environment:** `MAGICER_LOG_FORMAT`  
**Description:** Log output format.

**Valid Values:**
- `"json"` - Structured JSON (production)
- `"pretty"` - Human-readable (development)
- `"compact"` - Minimal console output

**Example:**
```toml
[logging]
format = "json"
```

---

## Complete Configuration Example

### Minimal Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[sandbox]
base_dir = "/var/lib/magicer/files"

[auth]
username = "api_user"
# Use MAGICER_AUTH_PASSWORD environment variable
```

### Full Configuration with All Options

```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
backlog = 1024

[server.timeouts]
read_timeout_secs = 60
write_timeout_secs = 60
analysis_timeout_secs = 30
keepalive_secs = 75

[server.limits]
max_body_size_mb = 100
max_uri_length = 8192
max_header_size = 16384

[analysis]
large_file_threshold_mb = 10
write_buffer_size_kb = 64
temp_dir = "/tmp/magicer"

[sandbox]
base_dir = "/var/lib/magicer/files"

[auth]
username = "api_user"
password = "secret_password"  # Use env var in production

[magic]
database_path = "/usr/share/misc/magic.mgc"

[logging]
level = "info"
format = "json"
```

### Development Configuration

```toml
[server]
host = "127.0.0.1"
port = 8080
max_connections = 100

[server.timeouts]
read_timeout_secs = 120
analysis_timeout_secs = 60

[analysis]
large_file_threshold_mb = 5  # Lower threshold for testing
write_buffer_size_kb = 32
temp_dir = "./tmp"

[sandbox]
base_dir = "./test-files"

[auth]
username = "dev"
password = "dev"

[logging]
level = "debug"
format = "pretty"
```

### Production Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 2000
backlog = 2048

[server.timeouts]
read_timeout_secs = 30
write_timeout_secs = 30
analysis_timeout_secs = 20
keepalive_secs = 60

[server.limits]
max_body_size_mb = 100

[analysis]
large_file_threshold_mb = 10
write_buffer_size_kb = 128  # Optimized for SSD
temp_dir = "/var/tmp/magicer"

[sandbox]
base_dir = "/srv/magicer/files"

[auth]
username = "api_production"
# Password from MAGICER_AUTH_PASSWORD environment variable

[logging]
level = "info"
format = "json"
```

---

## Environment Variable Reference

Complete mapping of environment variables to configuration keys.

| Environment Variable | Configuration Key | Type | Default |
|---------------------|------------------|------|---------|
| `MAGICER_CONFIG_PATH` | N/A (file location) | String | `./config/config.toml` or `/etc/magicer/config.toml` |
| `MAGICER_HOST` | `server.host` | String | `"0.0.0.0"` |
| `MAGICER_PORT` | `server.port` | u16 | `8080` |
| `MAGICER_SANDBOX_DIR` | `sandbox.base_dir` | String | `/var/lib/magicer/files` |
| `MAGICER_AUTH_USERNAME` | `auth.username` | String | None (required) |
| `MAGICER_AUTH_PASSWORD` | `auth.password` | String | None (required) |
| `MAGICER_LOG_FORMAT` | `logging.format` | String | `"json"` |
| `RUST_LOG` | `logging.level` | String | `"info"` |

---

## Configuration Validation

The server validates configuration on startup:

**Required Fields:**
- `auth.username` (or `MAGICER_AUTH_USERNAME`)
- `auth.password` (or `MAGICER_AUTH_PASSWORD`)
- `sandbox.base_dir` must exist as a directory

**Validation Checks:**
- Port number in valid range (1-65535)
- Timeout values > 0
- Size limits > 0
- Paths must be absolute and accessible

**Startup Failure:**
If configuration is invalid, the server exits with an error message indicating the problem.

---

## Security Best Practices

1. **Never commit credentials** - Use environment variables for `auth.password`
2. **Restrict file permissions** - Set `config.toml` to mode 0600 (owner read/write only)
3. **Use absolute paths** - Always use absolute paths for `sandbox.base_dir`
4. **Validate sandbox** - Ensure sandbox directory exists before starting server
5. **Monitor timeouts** - Adjust timeouts based on actual file analysis performance
6. **Limit connections** - Set `max_connections` based on available system resources

---

## Troubleshooting

### Configuration File Not Found

**Error:** `Failed to load configuration file`

**Solution:**
- Check file exists at default location
- Use `MAGICER_CONFIG_PATH` to specify custom location
- Verify file permissions (must be readable by server process)

### Invalid Configuration Value

**Error:** `Configuration validation failed`

**Solution:**
- Check all required fields are present
- Verify value types match specification
- Ensure numeric values are within valid ranges

### Sandbox Directory Issues

**Error:** `Sandbox directory does not exist` or `Permission denied`

**Solution:**
- Create directory: `mkdir -p /var/lib/magicer/files`
- Set permissions: `chown magicer:magicer /var/lib/magicer/files`
- Verify path is absolute, not relative

### Port Already in Use

**Error:** `Address already in use`

**Solution:**
- Change `server.port` to different value
- Stop conflicting process using the port
- Use reverse proxy on port 80/443

---

## Related Documentation

- [Deployment Guide](../how-to-guides/DEPLOYMENT.md) - Production deployment with configuration
- [HTTP Server Specification](HTTP_SERVER.md) - Server behavior and limits
- [Architecture Design](../explanation/ARCHITECTURE.md) - Configuration loading strategy
