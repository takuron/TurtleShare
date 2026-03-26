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
secret = "your-secret-key-change-in-production"
expiry_hours = 24
rotation_days = 30  # Auto-rotate JWT secret every N days

[site_info]
name = "TurtleShare"  # Site name / 网站名称
author = "Admin"  # Site author / 网站作者
sponsor_link = ""  # Sponsor link (optional) / 赞助链接（可选）
header_image = ""  # Site header image path (optional) / 网站头图路径（可选）
```

## Config Structure / 配置结构

```rust
pub struct Config {
    pub admin: AdminConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub jwt: JwtConfig,
    pub site_info: SiteInfoConfig,
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

pub struct SiteInfoConfig {
    pub name: String,
    pub author: String,
    pub sponsor_link: Option<String>,
    pub header_image: Option<String>,
}
```
