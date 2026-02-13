use magicer::infrastructure::config::server_config::ServerConfig;
use magicer::domain::errors::ValidationError;
use std::env;
use std::fs;
use serial_test::serial;

#[test]
#[serial]
fn test_default_config_values() {
    let config = ServerConfig::default();
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.analysis.large_file_threshold_mb, 10);
    assert_eq!(config.sandbox.base_dir, "/tmp/magicer/files");
}

#[test]
#[serial]
fn test_load_from_toml() {
    let test_toml = "test_config_load.toml";
    let content = r#"
[server]
host = "192.168.1.1"
port = 9090

[analysis]
large_file_threshold_mb = 5
"#;
    fs::write(test_toml, content).unwrap();
    
    let config = ServerConfig::load(Some(test_toml.to_string()));
    
    assert_eq!(config.server.host, "192.168.1.1");
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.analysis.large_file_threshold_mb, 5);
    // Missing fields should be default
    assert_eq!(config.analysis.write_buffer_size_kb, 64);
    
    fs::remove_file(test_toml).unwrap();
}

#[test]
#[serial]
fn test_env_overrides() {
    // Clear potentially conflicting env vars
    env::remove_var("MAGICER_HOST");
    env::remove_var("MAGICER_PORT");
    env::remove_var("HOST");
    env::remove_var("PORT");
    
    env::set_var("MAGICER_HOST", "0.0.0.0");
    env::set_var("MAGICER_PORT", "7070");
    
    // Ensure we don't load from a default file if it exists
    env::set_var("MAGICER_CONFIG_PATH", "/non/existent/config.toml");
    
    let config = ServerConfig::load(None);
    
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 7070);
    
    env::remove_var("MAGICER_HOST");
    env::remove_var("MAGICER_PORT");
    env::remove_var("MAGICER_CONFIG_PATH");
}

#[test]
#[serial]
fn test_validate_creates_directories() {
    let mut config = ServerConfig::default();
    let temp_base = env::temp_dir().join("magicer_test_dir_create");
    let sandbox_dir = temp_base.join("files");
    let temp_analysis_dir = temp_base.join("analysis");
    
    // Ensure they don't exist
    if temp_base.exists() {
        fs::remove_dir_all(&temp_base).unwrap();
    }
    
    config.sandbox.base_dir = sandbox_dir.to_str().unwrap().to_string();
    config.analysis.temp_dir = temp_analysis_dir.to_str().unwrap().to_string();
    
    assert!(config.validate().is_ok());
    assert!(sandbox_dir.exists());
    assert!(temp_analysis_dir.exists());
    
    fs::remove_dir_all(temp_base).unwrap();
}

#[test]
#[serial]
fn test_validate_invalid_host_returns_error() {
    let mut config = ServerConfig::default();
    config.server.host = "".to_string();
    
    let result = config.validate();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ValidationError::EmptyValue));
}
