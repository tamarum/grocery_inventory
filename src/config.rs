use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub shopping: ShoppingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShoppingConfig {
    #[serde(default = "default_threshold")]
    pub low_stock_threshold: u32,
    #[serde(default = "default_true")]
    pub include_out_of_stock: bool,
}

impl Default for ShoppingConfig {
    fn default() -> Self {
        Self {
            low_stock_threshold: default_threshold(),
            include_out_of_stock: true,
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_threshold() -> u32 {
    2
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_config() {
        let toml_str = r#"
[database]
path = "test.db"

[web]
host = "0.0.0.0"
port = 8080

[shopping]
low_stock_threshold = 3
include_out_of_stock = false
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.database.path, PathBuf::from("test.db"));
        assert_eq!(config.web.port, 8080);
        assert_eq!(config.shopping.low_stock_threshold, 3);
        assert!(!config.shopping.include_out_of_stock);
    }

    #[test]
    fn defaults_applied() {
        let toml_str = r#"
[database]
path = "test.db"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.web.host, "127.0.0.1");
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.shopping.low_stock_threshold, 2);
        assert!(config.shopping.include_out_of_stock);
    }
}
