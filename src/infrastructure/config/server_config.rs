use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub analysis: AnalysisConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnalysisConfig {
    pub large_file_threshold_mb: usize,
    pub write_buffer_size_kb: usize,
    pub temp_dir: String,
    pub min_free_space_mb: usize,
}

impl ServerConfig {
    pub fn load_from_env() -> Self {
        // Default values
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            analysis: AnalysisConfig {
                large_file_threshold_mb: env::var("ANALYSIS_LARGE_FILE_THRESHOLD_MB")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                write_buffer_size_kb: env::var("ANALYSIS_WRITE_BUFFER_SIZE_KB")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(64),
                temp_dir: env::var("ANALYSIS_TEMP_DIR").unwrap_or_else(|_| "/tmp/magicer".to_string()),
                min_free_space_mb: env::var("ANALYSIS_MIN_FREE_SPACE_MB")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1024),
            },
        }
    }
}
