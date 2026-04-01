# Configuration / 配置文件

## config.toml

```toml
[admin]
username = "admin"
password_hash = "$argon2id$..."  # Argon2 hash

[server]
host = "127.0.0.1"
port = 3000
base_url = "https://example.com"  # Base URL for the site / 网站基础 URL

[database]
path = "./data/turtleshare.db"

[storage]
static_path = "./static"  # Static files directory (frontend) / 静态文件目录（前端）
files_path = "./data/files"
max_upload_size_mb = 1024  # Maximum upload file size in MB / 最大上传文件大小（MB）

[jwt]
base_secret = "your-secret-key-change-in-production"  # Base secret for generating rotating JWT secrets / 用于生成轮换 JWT 密钥的基础密钥
expiry_hours = 24
rotation_days = 30  # Auto-rotate JWT secret every N days

[hashid]
min_length = 6  # Minimum length of encoded user IDs / 编码用户 ID 的最小长度

[siteinfo]
# All keys under [siteinfo] are forwarded verbatim to GET /api/public/site-info.
# Add any key-value pairs your frontend requires — no code changes needed.
# Supported TOML value types: string, integer, float, boolean, array, inline table.
#
# [siteinfo] 下的所有键值对将原样转发至 GET /api/public/site-info。
# 可根据前端需要添加任意键值，无需修改代码。
# 支持的 TOML 值类型：字符串、整数、浮点数、布尔值、数组、内联表。
name = "TurtleShare"
author = "Admin"
sponsor_link = ""
header_image = ""
# Example custom fields / 自定义字段示例:
# theme_color = "#3498db"
# show_upload_button = true
# custom_footer = "Powered by TurtleShare"
```

## Config Structure / 配置结构

```rust
pub struct Config {
    pub admin: AdminConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub jwt: JwtConfig,
    pub hashid: HashIdConfig,
    pub siteinfo: SiteInfoConfig,  // type alias for toml::Table
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

pub struct StorageConfig {
    pub files_path: String,
    pub max_upload_size_mb: u64,
    pub static_path: String,  // Defaults to "./static"
}

// SiteInfoConfig is a type alias for toml::Table (free-form key-value store).
// All entries are serialized as-is to JSON for the public site-info endpoint.
//
// SiteInfoConfig 是 toml::Table 的类型别名（自由格式键值存储）。
// 所有条目将原样序列化为 JSON，通过公开站点信息端点返回。
pub type SiteInfoConfig = toml::Table;
```
