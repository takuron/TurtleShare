# Configuration / 配置文件

## config.toml

```toml
[admin]
username = "admin"  # Admin login username / 管理员登录用户名
password_hash = "$argon2id$..."  # Argon2id password hash, not raw password / Argon2id 密码哈希，不是明文密码

[server]
host = "127.0.0.1"  # Bind address for the HTTP server / HTTP 服务监听地址
port = 3000  # Bind port for the HTTP server / HTTP 服务监听端口
base_url = "https://example.com"  # Public base URL used in generated links and file URLs / 对外访问的站点基础 URL，用于生成链接和文件 URL

[database]
path = "./data/turtleshare.db"  # SQLite database file path / SQLite 数据库文件路径

[storage]
static_path = "./static"  # Frontend static assets directory, defaults to "./static" if omitted / 前端静态资源目录，省略时默认为 "./static"
files_path = "./data/files"  # Uploaded file storage directory / 上传文件存储目录
max_upload_size_mb = 1024  # Maximum upload size in megabytes / 单个上传文件大小上限（MB）

[jwt]
base_secret = "your-secret-key-change-in-production"  # Base secret used to derive rotating JWT signing keys / 用于派生轮换 JWT 签名密钥的基础密钥
expiry_hours = 24  # JWT token lifetime in hours / JWT 令牌有效期（小时）
rotation_days = 30  # Rotate JWT secret every N days / JWT 密钥每 N 天轮换一次

[hashid]
min_length = 6  # Minimum length of encoded external IDs, defaults to 6 / 对外 HashID 的最小长度，省略时默认为 6

[siteinfo]
# All keys under [siteinfo] are forwarded verbatim to GET /api/public/site-info.
# [siteinfo] 下的所有键值都会原样转发到 GET /api/public/site-info。
#
# Add any frontend-facing metadata you need here without changing backend code.
# 可在此添加任意前端需要的站点元数据，无需修改后端代码。
#
# Supported TOML value types: string, integer, float, boolean, array, inline table, nested table.
# 支持的 TOML 值类型：字符串、整数、浮点数、布尔值、数组、内联表、嵌套表。
name = "TurtleShare"  # Site or creator display name / 站点或创作者显示名称
author = "Admin"  # Author or operator name / 作者或运营者名称
avatar = ""  # Public avatar image URL / 公开头像图片 URL
bio = "Admin"  # Public profile or introduction text / 公开简介文本

[[siteinfo.social_links]]
platform = "github"  # Social platform identifier / 社交平台标识
url = "https://github.com/example"  # Public profile URL for this platform / 该平台对应的公开资料链接

[[siteinfo.social_links]]
platform = "x"  # Social platform identifier / 社交平台标识
url = "https://x.com/example"  # Public profile URL for this platform / 该平台对应的公开资料链接

[[siteinfo.social_links]]
platform = "bilibili"  # Social platform identifier / 社交平台标识
url = "https://space.bilibili.com/12345"  # Public profile URL for this platform / 该平台对应的公开资料链接

[[siteinfo.social_links]]
platform = "telegram"  # Social platform identifier / 社交平台标识
url = "https://t.me/example"  # Public profile URL for this platform / 该平台对应的公开资料链接

[[siteinfo.social_links]]
platform = "email"  # Social platform identifier / 社交平台标识
url = "mailto:hello@example.com"  # Contact URL such as mailto: / 联系方式链接，例如 mailto:
```

## Option Reference / 配置项说明

### `[admin]`

- `username`: The administrator username used by `POST /api/admin/login`. / 管理员登录用户名，用于 `POST /api/admin/login`。
- `password_hash`: Argon2id hash of the admin password. Generate it with `cargo run -- hash-pw your-password`. / 管理员密码的 Argon2id 哈希值，可通过 `cargo run -- hash-pw your-password` 生成。

### `[server]`

- `host`: Network interface address to bind the HTTP server to. Use `127.0.0.1` behind a reverse proxy, or `0.0.0.0` if you want it reachable from other machines. / HTTP 服务监听地址；如果前面有反向代理可使用 `127.0.0.1`，需要局域网或外网直接访问时可使用 `0.0.0.0`。
- `port`: TCP port for the HTTP server. / HTTP 服务监听端口。
- `base_url`: Publicly reachable base URL of the site. It is used when the backend generates external links, especially file URLs. / 站点对外可访问的基础 URL，后端在生成外部链接时会使用它，尤其是文件 URL。

### `[database]`

- `path`: Filesystem path to the SQLite database file. The parent directory is created automatically on startup when needed. / SQLite 数据库文件路径；如有需要，启动时会自动创建父目录。

### `[storage]`

- `static_path`: Directory used to serve frontend static assets. If omitted, it defaults to `./static`. / 用于提供前端静态资源的目录；省略时默认值为 `./static`。
- `files_path`: Directory used to store uploaded files on disk. / 用于在本地磁盘上保存上传文件的目录。
- `max_upload_size_mb`: Maximum allowed upload size in megabytes. Requests exceeding this limit are rejected. / 允许上传的单文件最大体积，单位为 MB；超过限制的请求会被拒绝。

### `[jwt]`

- `base_secret`: Root secret used to derive rotating JWT signing keys. This should be long, random, and kept private in production. / 用于派生轮换 JWT 签名密钥的根密钥；生产环境中应使用足够长且随机的私密值。
- `expiry_hours`: Token validity duration in hours. / 令牌有效期，单位为小时。
- `rotation_days`: Interval in days for rotating the JWT secret pair stored by the server. / 服务端轮换 JWT 密钥对的时间间隔，单位为天。

### `[hashid]`

- `min_length`: Minimum length for encoded external IDs such as user-facing hash IDs. If omitted, the default is `6`. / 对外编码 ID 的最小长度，例如用户可见的 HashID；省略时默认值为 `6`。

### `[siteinfo]`

- `[siteinfo]` is a free-form table. Every key under it is forwarded as JSON by `GET /api/public/site-info`. / `[siteinfo]` 是自由结构的配置表，其中的所有键都会通过 `GET /api/public/site-info` 以 JSON 原样返回。
- Use it for frontend-visible metadata such as site name, profile text, avatars, links, theme settings, or any custom structured data. / 可用于前端展示的站点名称、简介、头像、链接、主题设置或任意自定义结构化数据。
- `social_links` in the example is just a conventional array-of-tables, not a hardcoded schema. / 示例中的 `social_links` 只是一个常见的数组表写法，并不是写死的固定 schema。

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

pub struct AdminConfig {
    pub username: String,
    pub password_hash: String,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

pub struct DatabaseConfig {
    pub path: String,
}

pub struct StorageConfig {
    pub files_path: String,
    pub max_upload_size_mb: u64,
    pub static_path: String,  // Defaults to "./static"
}

pub struct JwtConfig {
    pub base_secret: String,
    pub expiry_hours: u64,
    pub rotation_days: u64,
}

pub struct HashIdConfig {
    pub min_length: usize,  // Defaults to 6
}

// SiteInfoConfig is a type alias for toml::Table (free-form key-value store).
// All entries are serialized as-is to JSON for the public site-info endpoint.
//
// SiteInfoConfig 是 toml::Table 的类型别名（自由格式键值存储）。
// 所有条目都会原样序列化为 JSON，并通过公开的 site-info 接口返回。
pub type SiteInfoConfig = toml::Table;
```
