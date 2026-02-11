use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub server: ServerSection,
    pub analysis: AnalysisConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnalysisConfig {
    pub large_file_threshold_mb: usize,
    pub write_buffer_size_kb: usize,
    pub temp_dir: String,
    pub min_free_space_mb: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            analysis: AnalysisConfig {
                large_file_threshold_mb: 10,
                write_buffer_size_kb: 64,
                temp_dir: "/tmp/magicer".to_string(),
                min_free_space_mb: 1024,
            },
        }
    }
}

impl ServerConfig {
    pub fn load() -> Self {
        let mut config = Self::load_from_toml().unwrap_or_default();
        config.apply_env_overrides();
        config
    }

    fn load_from_toml() -> Option<Self> {
        let config_path = env::var("MAGICER_CONFIG_PATH")
            .unwrap_or_else(|_| "config/config.toml".to_string());

        fs::read_to_string(config_path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(host) = env::var("MAGICER_HOST").or_else(|_| env::var("HOST")) {
            self.server.host = host;
        }
        if let Ok(port) = env::var("MAGICER_PORT").or_else(|_| env::var("PORT")) {
            if let Ok(port) = port.parse() {
                self.server.port = port;
            }
        }
        if let Ok(val) = env::var("ANALYSIS_LARGE_FILE_THRESHOLD_MB") {
            if let Ok(val) = val.parse() {
                self.analysis.large_file_threshold_mb = val;
            }
        }
        if let Ok(val) = env::var("ANALYSIS_WRITE_BUFFER_SIZE_KB") {
            if let Ok(val) = val.parse() {
                self.analysis.write_buffer_size_kb = val;
            }
        }
        if let Ok(val) = env::var("ANALYSIS_TEMP_DIR") {
            self.analysis.temp_dir = val;
        }
        if let Ok(val) = env::var("ANALYSIS_MIN_FREE_SPACE_MB") {
            if let Ok(val) = val.parse() {
                self.analysis.min_free_space_mb = val;
            }
        }
    }

    // Keep this for backward compatibility if needed, but updated to use new structure
    pub fn load_from_env() -> Self {
        Self::load()
    }
}
