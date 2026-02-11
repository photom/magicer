use magicer::infrastructure::config::server_config::ServerConfig;
use std::env;

#[test]
fn test_config_defaults() {
    // Ensure no env vars interfere
    env::remove_var("HOST");
    env::remove_var("PORT");
    env::remove_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB");
    
    let config = ServerConfig::load_from_env();
    
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert_eq!(config.analysis.large_file_threshold_mb, 10);
    assert_eq!(config.analysis.write_buffer_size_kb, 64);
}

#[test]
fn test_config_env_overrides() {
    env::set_var("HOST", "0.0.0.0");
    env::set_var("PORT", "8080");
    env::set_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB", "50");
    
    let config = ServerConfig::load_from_env();
    
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert_eq!(config.analysis.large_file_threshold_mb, 50);
    
    // Cleanup
    env::remove_var("HOST");
    env::remove_var("PORT");
    env::remove_var("ANALYSIS_LARGE_FILE_THRESHOLD_MB");
}
