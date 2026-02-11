use magicer::infrastructure::config::server_config::ServerConfig;
use std::env;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_startup_invalid_port_fails() {
    let mut config = ServerConfig::default();
    config.server.port = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_startup_sandbox_missing_fails() {
    let mut config = ServerConfig::default();
    config.sandbox.base_dir = "/non_existent_directory_magicer_test".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_startup_valid_config_passes() {
    let dir = tempdir().unwrap();
    let mut config = ServerConfig::default();
    config.sandbox.base_dir = dir.path().to_str().unwrap().to_string();
    config.auth.username = "admin".to_string();
    config.auth.password = "password".to_string();
    assert!(config.validate().is_ok());
}
