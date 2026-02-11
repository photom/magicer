# Test Plan: Configuration Loading

This test plan covers the hierarchical configuration loading mechanism, ensuring that the application correctly prioritizes environment variables, TOML files, and default values.

## test_config_default_values

**Setup:**
- Clear all relevant environment variables (`MAGICER_*`, `HOST`, `PORT`, `ANALYSIS_*`).
- Ensure no `config/config.toml` exists in the execution path (or point `MAGICER_CONFIG_PATH` to a non-existent file).

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- `server.host` is "127.0.0.1".
- `server.port` is 3000.
- `analysis.large_file_threshold_mb` is 10.
- `analysis.write_buffer_size_kb` is 64.
- `analysis.temp_dir` is "/tmp/magicer".
- `analysis.min_free_space_mb` is 1024.

## test_config_toml_loading

**Setup:**
- Create a temporary TOML file with specific values:
  ```toml
  [server]
  host = "10.0.0.1"
  port = 9000
  [analysis]
  large_file_threshold_mb = 20
  ```
- Set `MAGICER_CONFIG_PATH` to the path of this temporary file.

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- `server.host` is "10.0.0.1".
- `server.port` is 9000.
- `analysis.large_file_threshold_mb` is 20.
- Other values (not in TOML) remain at their defaults.

## test_config_env_priority_over_toml

**Setup:**
- Create a temporary TOML file with `server.port = 9000`.
- Set `MAGICER_CONFIG_PATH` to this file.
- Set environment variable `MAGICER_PORT = 9999`.

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- `server.port` is 9999 (Environment variable takes priority).

## test_config_env_legacy_support

**Setup:**
- Clear `MAGICER_HOST` and `MAGICER_PORT`.
- Set legacy environment variables `HOST = "0.0.0.0"` and `PORT = "8080"`.

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- `server.host` is "0.0.0.0".
- `server.port` is 8080.

## test_config_invalid_toml_fallback_to_default

**Setup:**
- Create a temporary TOML file with invalid syntax.
- Set `MAGICER_CONFIG_PATH` to this file.

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- The function does not panic.
- The returned configuration uses default values.

## test_config_custom_path_override

**Setup:**
- Create two TOML files: `default_path.toml` and `custom_path.toml` with different port values.
- Place `default_path.toml` at `config/config.toml`.
- Set `MAGICER_CONFIG_PATH` to `custom_path.toml`.

**Execution:**
- Call `ServerConfig::load()`.

**Assertions:**
- The configuration is loaded from `custom_path.toml`.
