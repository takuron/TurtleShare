use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::{AppError, Result};

/// The root configuration structure for TurtleShare.
///
/// This structure mirrors the `config.toml` file format.
//
// // TurtleShare 的根配置结构。
// //
// // 此结构与 `config.toml` 文件格式相对应。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub admin: AdminConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub jwt: JwtConfig,
    pub hashid: HashIdConfig,
    pub site_info: SiteInfoConfig,
}

/// Administrator configuration.
//
// // 管理员配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminConfig {
    pub username: String,
    pub password_hash: String,
}

/// Server network configuration.
//
// // 服务器网络配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

/// Database configuration.
//
// // 数据库配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

/// File storage configuration.
//
// // 文件存储配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub files_path: String,
    pub max_upload_size_mb: u64,
    #[serde(default = "default_static_path")]
    pub static_path: String,
}

fn default_static_path() -> String {
    "./static".to_string()
}

/// JWT authentication configuration.
//
// // JWT 身份验证配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    pub base_secret: String,
    pub expiry_hours: u64,
    pub rotation_days: u64,
}

/// HashID configuration for encoding user IDs.
//
// // HashID 配置，用于编码用户 ID。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashIdConfig {
    #[serde(default = "default_hashid_min_length")]
    pub min_length: usize,
}

fn default_hashid_min_length() -> usize {
    6
}

/// Public site information.
//
// // 公开站点信息。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteInfoConfig {
    pub name: String,
    pub author: String,
    pub sponsor_link: Option<String>,
    pub header_image: Option<String>,
    pub base_url: String,
}

impl Config {
    /// Loads the configuration from a specified TOML file.
    ///
    /// # Arguments
    /// * `path` - The path to the configuration file.
    ///
    /// # Returns
    /// Returns the parsed `Config` or an error if the file cannot be read or parsed.
    //
    // // 从指定的 TOML 文件加载配置。
    // //
    // // # 参数
    // // * `path` - 配置文件路径。
    // //
    // // # 返回
    // // 返回解析后的 `Config`，如果文件无法读取或解析，则返回错误。
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        // 1. 读取配置文件内容。
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        // 2. 将 TOML 内容反序列化为 Config 结构。
        let config: Config = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }
}
