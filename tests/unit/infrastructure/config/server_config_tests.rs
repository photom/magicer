use magicer::infrastructure::config::server_config::ServerConfig;
use std::env;
use std::fs;

#[test]
fn test_config_defaults() {
    // Ensure no env vars interfere
    env::remove_var("HOST");
    env::remove_var("PORT");
    env::remove_var("MAGICER_HOST");
    env::remove_var("MAGICER_PORT");
    env::remove_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB");
    env::remove_var("MAGICER_CONFIG_PATH");
    
    // We expect it to try loading from config/config.toml if it exists.
    // For this test to be truly about defaults, we'd need to ensure config/config.toml is NOT there, 
    // OR we point MAGICER_CONFIG_PATH to a non-existent file.
    env::set_var("MAGICER_CONFIG_PATH", "non_existent.toml");
    
    let config = ServerConfig::load();
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.analysis.large_file_threshold_mb, 10);
    assert_eq!(config.analysis.write_buffer_size_kb, 64);
    
    env::remove_var("MAGICER_CONFIG_PATH");
}

#[test]
fn test_config_env_overrides() {
    env::set_var("MAGICER_HOST", "0.0.0.0");
    env::set_var("MAGICER_PORT", "8080");
    env::set_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB", "50");
    env::set_var("MAGICER_CONFIG_PATH", "non_existent.toml");
    
    let config = ServerConfig::load();
    
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
    
    let config = ServerConfig::load();
    
    assert_eq!(config.server.host, "10.0.0.1");
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.analysis.large_file_threshold_mb, 20);
    assert_eq!(config.analysis.temp_dir, "/tmp/test");
    
    // Cleanup
    fs::remove_file(test_toml).unwrap();
    env::remove_var("MAGICER_CONFIG_PATH");
}
