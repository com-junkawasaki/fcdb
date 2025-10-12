//! Configuration management for Own-CFA-Enishi

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
    pub max_connections: usize,
    pub timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            workers: num_cpus::get(),
            max_connections: 10000,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub path: PathBuf,
    pub max_size_gb: u64,
    pub compression: bool,
    pub sync_writes: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data"),
            max_size_gb: 100,
            compression: true,
            sync_writes: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub query_cache_size: usize,
    pub bloom_filter_size: usize,
    pub max_concurrent_queries: usize,
    pub adaptive_optimization: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            query_cache_size: 1000000,
            bloom_filter_size: 10000000,
            max_concurrent_queries: 1000,
            adaptive_optimization: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_audit: bool,
    pub audit_log_path: PathBuf,
    pub max_sessions: usize,
    pub session_timeout_secs: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_audit: true,
            audit_log_path: PathBuf::from("./logs/audit.log"),
            max_sessions: 10000,
            session_timeout_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub enable_prometheus: bool,
    pub log_level: String,
    pub health_check_interval_secs: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_port: 9090,
            enable_prometheus: true,
            log_level: "info".to_string(),
            health_check_interval_secs: 30,
        }
    }
}

/// Load configuration from multiple sources
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::default();

    // Load from environment variables
    if let Ok(port) = env::var("ENISHI_PORT") {
        config.server.port = port.parse()?;
    }

    if let Ok(host) = env::var("ENISHI_HOST") {
        config.server.host = host;
    }

    if let Ok(storage_path) = env::var("ENISHI_STORAGE_PATH") {
        config.storage.path = PathBuf::from(storage_path);
    }

    if let Ok(log_level) = env::var("RUST_LOG") {
        config.monitoring.log_level = log_level;
    }

    // Load from config file if it exists
    let config_paths = [
        "enishi.toml",
        "/etc/enishi/config.toml",
        "./config/enishi.toml",
    ];

    for path in &config_paths {
        if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)?;
            let file_config: Config = toml::from_str(&content)?;
            config = merge_configs(config, file_config);
            break;
        }
    }

    // Validate configuration
    validate_config(&config)?;

    Ok(config)
}

/// Merge two configurations (file config overrides defaults)
fn merge_configs(base: Config, override_config: Config) -> Config {
    // For now, just return the override config
    // In a full implementation, you'd merge recursively
    override_config
}

/// Validate configuration values
fn validate_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.server.port == 0 {
        return Err("Invalid server port".into());
    }

    if config.storage.max_size_gb == 0 {
        return Err("Invalid storage size".into());
    }

    if config.performance.query_cache_size == 0 {
        return Err("Invalid cache size".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storage.max_size_gb, 100);
        assert!(config.performance.adaptive_optimization);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.server.port = 0;
        assert!(validate_config(&config).is_err());
    }
}
