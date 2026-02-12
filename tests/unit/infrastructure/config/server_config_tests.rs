use magicer::infrastructure::config::server_config::ServerConfig;
use magicer::domain::errors::ValidationError;
use std::env;
use std::fs;
use serial_test::serial;

#[test]
#[serial]
fn test_config_defaults() {
    env::remove_var("HOST");
    env::remove_var("PORT");
    env::remove_var("MAGICER_HOST");
    env::remove_var("MAGICER_PORT");
    env::remove_var("MAGICER_AUTH_USERNAME");
    env::remove_var("MAGICER_AUTH_PASSWORD");
    env::remove_var("MAGICER_SANDBOX_DIR");
    env::remove_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB");
    env::remove_var("MAGICER_CONFIG_PATH");
    env::remove_var("MAGICER_LOG_LEVEL");
    
    // We expect it to try loading from config/config.toml if it exists.
    // For this test to be truly about defaults, we'd need to ensure config/config.toml is NOT there, 
    // OR we point MAGICER_CONFIG_PATH to a non-existent file.
    env::set_var("MAGICER_CONFIG_PATH", "non_existent.toml");
    
    let config = ServerConfig::load(None);
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.analysis.large_file_threshold_mb, 10);
    assert_eq!(config.analysis.write_buffer_size_kb, 64);
    
    env::remove_var("MAGICER_CONFIG_PATH");
}

#[test]
#[serial]
fn test_config_env_overrides() {
    env::set_var("MAGICER_HOST", "0.0.0.0");
    env::set_var("MAGICER_PORT", "8080");
    env::set_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB", "50");
    env::set_var("MAGICER_CONFIG_PATH", "non_existent.toml");
    
    let config = ServerConfig::load(None);
    
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.analysis.large_file_threshold_mb, 50);
    
    // Cleanup
    env::remove_var("MAGICER_HOST");
    env::remove_var("MAGICER_PORT");
    env::remove_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB");
    env::remove_var("MAGICER_CONFIG_PATH");
}

#[test]
#[serial]
fn test_config_toml_loading() {
    let test_toml = "test_config.toml";
    let content = r#"
[server]
host = "10.0.0.1"
port = 9000

[analysis]
large_file_threshold_mb = 20
write_buffer_size_kb = 128
temp_dir = "/tmp/test"
min_free_space_mb = 512
"#;
    fs::write(test_toml, content).unwrap();
    env::set_var("MAGICER_CONFIG_PATH", test_toml);
    
    let config = ServerConfig::load(None);
    
    assert_eq!(config.server.host, "10.0.0.1");
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.analysis.large_file_threshold_mb, 20);
    assert_eq!(config.analysis.temp_dir, "/tmp/test");
    
    // Cleanup
    fs::remove_file(test_toml).unwrap();
    env::remove_var("MAGICER_CONFIG_PATH");
}

#[test]
fn test_validate_success() {
    let mut config = ServerConfig::default();
    // Ensure sandbox dir exists
    let temp_dir = std::env::temp_dir().join("magicer_test_success");
    fs::create_dir_all(&temp_dir).unwrap();
    config.sandbox.base_dir = temp_dir.to_str().unwrap().to_string();
    
    assert!(config.validate().is_ok());
    
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_validate_missing_sandbox_dir() {
    let mut config = ServerConfig::default();
    config.sandbox.base_dir = "/non/existent/path/definitely".to_string();
    
    let result = config.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ValidationError::FileNotFound);
}

#[test]
fn test_validate_empty_host() {
    let mut config = ServerConfig::default();
    config.server.host = "".to_string();
    // Ensure sandbox exists so we don't fail on that
    let temp_dir = std::env::temp_dir().join("magicer_test_host");
    fs::create_dir_all(&temp_dir).unwrap();
    config.sandbox.base_dir = temp_dir.to_str().unwrap().to_string();

    let result = config.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ValidationError::EmptyValue);
    
    fs::remove_dir_all(temp_dir).unwrap();
}
