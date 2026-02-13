# Test Plan: ServerConfig

## test_default_config_values

**Setup:**
- None.

**Execution:**
- Call `ServerConfig::default()`.

**Assertions:**
- `server.host` is "127.0.0.1".
- `server.port` is 8080.
- `analysis.large_file_threshold_mb` is 10.
- `sandbox.base_dir` is "/tmp/magicer/files".

## test_load_from_toml

**Setup:**
- Create a temporary TOML file with specific values.

**Execution:**
- Call `ServerConfig::load(Some(temp_file_path))`.

**Assertions:**
- Values from TOML are correctly loaded.
- Missing values in TOML are filled with defaults.

## test_env_overrides

**Setup:**
- Set `MAGICER_HOST` and `MAGICER_PORT` environment variables.

**Execution:**
- Call `ServerConfig::load(None)`.

**Assertions:**
- `server.host` and `server.port` match the environment variables.

## test_validate_creates_directories

**Setup:**
- `ServerConfig` with non-existent directory paths.

**Execution:**
- Call `config.validate()`.

**Assertions:**
- Result is `Ok`.
- The specified directories are created on the filesystem.

## test_validate_invalid_host_returns_error

**Setup:**
- `ServerConfig` with an empty host string.

**Execution:**
- Call `config.validate()`.

**Assertions:**
- Result is `Err(ValidationError::EmptyValue)`.
