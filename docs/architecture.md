# TurtleShare Architecture Document
# TurtleShare 架构文档

## 1. Overview / 概述

TurtleShare is a single-administrator membership content distribution platform backend, similar to Afdian/Patreon. Built with Rust, using Axum web framework and SQLite database.

TurtleShare 是一个单管理员的会员内容分发平台后端，类似于爱发电/Patreon。使用 Rust 构建，采用 Axum 网络框架和 SQLite 数据库。

### Core Features / 核心功能
- Admin authentication / 管理员鉴权
- Article publishing (Markdown) with file attachments / 文章发布（Markdown）及附件上传
- User management / 用户管理
- Subscription tiers and time-based access control / 订阅等级和基于时间的访问控制
- Content access authentication / 内容访问鉴权
- Local file storage with random path protection / 本地文件存储（随机路径保护）

## 2. Technology Stack / 技术栈

- **Web Framework**: Axum
- **Database**: SQLite with sqlx
- **Configuration**: TOML (config.toml)
- **Authentication**: JWT tokens
- **File Storage**: Local filesystem with UUID-based paths

## 3. Project Structure / 项目结构

```
src/
├── main.rs              # Entry point / 入口
├── config.rs            # TOML configuration / 配置读取
├── models/              # Data models / 数据模型
│   ├── mod.rs
│   ├── admin.rs         # Admin model / 管理员模型
│   ├── user.rs          # User model / 用户模型
│   ├── article.rs       # Article model / 文章模型
│   └── subscription.rs  # Subscription model / 订阅模型
├── db/                  # Database layer / 数据库层
│   ├── mod.rs
│   └── schema.sql       # Database schema / 数据库模式
├── handlers/            # HTTP handlers / HTTP 处理器
│   ├── mod.rs
│   ├── admin.rs         # Admin endpoints / 管理员端点
│   ├── user.rs          # User endpoints / 用户端点
│   ├── article.rs       # Article endpoints / 文章端点
│   └── file.rs          # File endpoints / 文件端点
├── middleware/          # Middleware / 中间件
│   ├── mod.rs
│   └── auth.rs          # Authentication / 鉴权
└── utils/               # Utilities / 工具函数
    ├── mod.rs
    ├── jwt.rs           # JWT utilities / JWT 工具
    └── file.rs          # File utilities / 文件工具
```

## 4. Configuration File / 配置文件

**config.toml**:
```toml
[admin]
username = "admin"
password_hash = "$argon2id$..."  # Argon2 hash

[server]
host = "127.0.0.1"
port = 3000

[database]
path = "./data/turtleshare.db"

[storage]
files_path = "./data/files"

[jwt]
secret = "your-secret-key-change-in-production"
expiry_hours = 24
rotation_days = 30  # Auto-rotate JWT secret every N days
```

## 5. Database Schema / 数据库模式

### users table / 用户表
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT,  -- 可选，管理员编辑，无验证
    note TEXT,   -- 可选，仅管理员可见
    created_at TEXT NOT NULL  -- RFC 3339: 2025-01-15T08:30:45.123+08:00
);
```

### user_subscriptions table / 用户订阅记录表
```sql
CREATE TABLE user_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    tier INTEGER NOT NULL,
    start_date TEXT NOT NULL,  -- RFC 3339: 2025-01-15T08:30:45.123+08:00
    end_date TEXT NOT NULL,    -- RFC 3339: 2025-01-15T08:30:45.123+08:00
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
-- 查询时：对于某个时间点，取所有覆盖该时间的订阅记录中的最高等级
```

### articles table / 文章表
```sql
CREATE TABLE articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    cover_image TEXT,  -- 头图 URL/path
    content TEXT NOT NULL,  -- Markdown
    required_tier INTEGER NOT NULL DEFAULT 0,
    is_public INTEGER NOT NULL DEFAULT 0,  -- 0=private, 1=public
    file_links TEXT,  -- JSON array of file URLs/paths
    created_at TEXT NOT NULL,  -- RFC 3339: 2025-01-15T08:30:45.123+08:00
    updated_at TEXT NOT NULL
);
-- 公开文章：所有人可见标题和头图，tier=0 可见内容
```

### files table / 文件表
```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT UNIQUE NOT NULL,  -- UUID for directory name
    original_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TEXT NOT NULL  -- RFC 3339: 2025-01-15T08:30:45.123+08:00
);
-- 存储路径: {BASEDIR}/files/{uuid}/{original_name}
-- 无鉴权，知道链接即可访问
```

### kv_store table / KV存储表
```sql
CREATE TABLE kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL,  -- RFC 3339: 2025-01-15T08:30:45.123+08:00
    updated_at TEXT NOT NULL
);
-- 用于存储程序运行时数据，如 JWT 密钥轮换
```

## 6. API Endpoints / API 端点

### Admin Endpoints / 管理员端点

**Authentication / 鉴权**
- `POST /api/admin/login` - Admin login / 管理员登录

**Users / 用户管理**
- `GET /api/admin/users` - List all users / 列出所有用户
- `GET /api/admin/users/:id` - Get user detail / 获取用户详情
- `GET /api/admin/users/:id/tier?at=<timestamp>` - Get user tier at specific time / 查询特定时间用户等级
- `POST /api/admin/users` - Create user / 创建用户
- `PUT /api/admin/users/:id` - Update user / 更新用户
- `DELETE /api/admin/users/:id` - Delete user / 删除用户

**User Subscriptions / 用户订阅管理**
- `GET /api/admin/users/:id/subscriptions` - List user subscriptions / 列出用户订阅
- `POST /api/admin/users/:id/subscriptions` - Add subscription period / 添加订阅时段
- `PUT /api/admin/subscriptions/:id` - Update subscription / 更新订阅
- `DELETE /api/admin/subscriptions/:id` - Delete subscription / 删除订阅

**Articles / 文章管理**
- `GET /api/admin/articles` - List all articles / 列出所有文章
- `GET /api/admin/articles/:id` - Get article detail / 获取文章详情
- `POST /api/admin/articles` - Create article / 创建文章
- `PUT /api/admin/articles/:id` - Update article / 更新文章
- `DELETE /api/admin/articles/:id` - Delete article / 删除文章

**Files / 文件管理**
- `GET /api/admin/files` - List all files / 列出所有文件
- `GET /api/admin/files/:id` - Get file metadata / 获取文件元数据
- `POST /api/admin/files` - Upload file / 上传文件
- `DELETE /api/admin/files/:id` - Delete file / 删除文件

### User Endpoints / 用户端点
- `POST /api/users/login` - User login / 用户登录
- `PUT /api/users/password` - Change password / 修改密码
- `GET /api/articles` - List accessible articles / 列出可访问文章
- `GET /api/articles/:id` - Get article detail / 获取文章详情

### Public Endpoints / 公开端点
- `GET /api/public/articles` - List public articles (title+cover+tier) / 列出公开文章（标题+头图+等级）
- `GET /api/public/articles/:id` - Get public article detail / 获取公开文章详情（tier=0显示内容）
- `GET /files/:uuid/:filename` - Download file (no auth) / 下载文件（无鉴权）

## 7. JSON Response Format / JSON 响应格式

All API responses use JSON format with a consistent structure.

### Success Response / 成功响应
```json
{
  "success": true,
  "data": <response_data>,
  "message": "Optional success message"
}
```

### Error Response / 错误响应
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

### Common Error Codes / 常见错误码
- `UNAUTHORIZED` - 401, invalid or missing token
- `FORBIDDEN` - 403, insufficient permissions
- `NOT_FOUND` - 404, resource not found
- `VALIDATION_ERROR` - 400, invalid input
- `INTERNAL_ERROR` - 500, server error

### Response Examples / 响应示例

**Login Success**
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": 1,
      "username": "admin",
      "role": "admin"
    }
  }
}
```

**Get User Detail (Admin)**
```json
{
  "success": true,
  "data": {
    "id": 5,
    "username": "user123",
    "email": "user@example.com",
    "note": "VIP customer",
    "created_at": "2025-01-15T08:30:45.123+08:00"
  }
}
```

**Get User Tier at Time**
```json
{
  "success": true,
  "data": {
    "user_id": 5,
    "tier": 2,
    "at": "2025-02-15T10:00:00.000+08:00"
  }
}
```

**List Articles**
```json
{
  "success": true,
  "data": {
    "articles": [
      {
        "id": 1,
        "title": "Article Title",
        "cover_image": "/files/uuid-123/cover.jpg",
        "required_tier": 1,
        "is_public": true,
        "created_at": "2025-01-20T14:30:00.000+08:00"
      }
    ],
    "total": 1
  }
}
```

## 8. Authentication Flow / 鉴权流程
1. Admin sends credentials to `/api/admin/login`
2. Server validates against config.toml
3. Returns JWT token with `role: admin`
4. Admin includes token in `Authorization: Bearer <token>` header

### User Authentication / 用户鉴权
1. User sends credentials to `/api/users/login`
2. Server validates against database
3. Returns JWT token with `role: user, user_id: X`
4. User includes token in header for protected endpoints

### JWT Secret Rotation / JWT 密钥轮换

**KV Storage Keys / KV 存储键**
- `jwt_secret_current` - Current secret (base64, 256-bit) / 当前密钥
- `jwt_secret_previous` - Previous secret (base64, nullable) / 上一个密钥
- `jwt_secret_date` - Current secret creation time (RFC 3339) / 当前密钥创建时间

**Secret Generation / 密钥生成**
```
new_secret = SHA256(config.jwt.secret + random_uuid_v4())
```
- Seed from `config.toml` jwt.secret (read each time, not stored)
- Random UUID v4 generated on each rotation
- Derived secret stored in database

**Initialization / 初始化**
1. On first startup, check if `jwt_secret_current` exists
2. If not: generate `SHA256(config.jwt.secret + UUID)`, store as `jwt_secret_current`
3. Set `jwt_secret_date = now`

**Rotation Logic / 轮换逻辑**
1. On startup, check: `now - jwt_secret_date > rotation_days`
2. If true:
   - `jwt_secret_previous = jwt_secret_current`
   - `jwt_secret_current = SHA256(config.jwt.secret + new_UUID())`
   - `jwt_secret_date = now`

**Token Signing / 签发令牌**
- Always use `jwt_secret_current` to sign new tokens

**Token Verification / 验证令牌**
1. Try verify with `jwt_secret_current`
2. If failed and `jwt_secret_previous` exists, try verify with it
3. If both fail, return unauthorized

**Security Note / 安全说明**
- Both secrets assumed secure; if either leaks, rotate both immediately
- Previous secret kept for smooth transition during rotation period
- Previous secret valid until all old tokens expire (expiry_hours)

### Access Control / 访问控制

**Article Access / 文章访问**
- Public articles (`is_public=1`): All users can see title and cover_image
- Public + tier 0: All users can see full content
- Private or tier > 0:
  1. Query user_subscriptions where `start_date <= article.created_at <= end_date`
  2. Get max tier from overlapping subscriptions
  3. Check `max_tier >= article.required_tier`

**File Access / 文件访问**
- No authentication required
- Direct access via `/files/:uuid/:filename`
- Security through obscurity (random UUID paths)

## 8. File Storage Strategy / 文件存储策略

### Storage Path / 存储路径
- Files stored in `{BASEDIR}/files/{uuid}/{original_filename}`
- UUID v4 for directory name ensures unpredictable paths
- Original filename preserved in path
- No directory traversal possible

### Upload Flow / 上传流程
1. Admin uploads file via `POST /api/admin/files`
2. Generate UUID v4
3. Create directory `{BASEDIR}/files/{uuid}/`
4. Save file as `{BASEDIR}/files/{uuid}/{original_filename}`
5. Insert record into files table with uuid and original_name
6. Return file metadata including access URL: `/files/{uuid}/{original_filename}`

### Download Flow / 下载流程
1. User/anyone requests `GET /files/:uuid/:filename`
2. No authentication required
3. Serve file directly from `{BASEDIR}/files/{uuid}/{filename}`
4. Return 404 if not found

## 9. Implementation Phases / 实现阶段

### Phase 1: Foundation / 基础
- [ ] Config loading from TOML
- [ ] Database initialization and migrations
- [ ] JWT utilities

### Phase 2: Admin Features / 管理员功能
- [ ] Admin authentication
- [ ] Article CRUD
- [ ] File upload
- [ ] User creation and management

### Phase 3: User Features / 用户功能
- [ ] User authentication
- [ ] Article listing with access control
- [ ] Article detail view
- [ ] Public file serving (no auth)

### Phase 4: Polish / 完善
- [ ] Error handling
- [ ] Logging
- [ ] Input validation

## 10. Security Considerations / 安全考虑

- Passwords hashed with Argon2
- JWT tokens with expiration
- File paths randomized (UUID)
- SQL injection prevented by sqlx parameterized queries
- File size limits enforced
- CORS configured appropriately
- Rate limiting recommended for production

## 11. Data Models / 数据模型

### Config Structure / 配置结构
```rust
pub struct Config {
    pub admin: AdminConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub jwt: JwtConfig,
}
```

### Core Models / 核心模型
```rust
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub note: Option<String>,  // Admin only
    pub created_at: String,  // RFC 3339 with timezone
}

pub struct UserSubscription {
    pub id: i64,
    pub user_id: i64,
    pub tier: i32,
    pub start_date: String,  // RFC 3339 with timezone
    pub end_date: String,    // RFC 3339 with timezone
}

pub struct Article {
    pub id: i64,
    pub title: String,
    pub cover_image: Option<String>,
    pub content: String,
    pub required_tier: i32,
    pub is_public: bool,
    pub file_links: Option<String>,  // JSON array
    pub created_at: String,  // RFC 3339 with timezone
}

pub struct File {
    pub id: i64,
    pub uuid: String,
    pub original_name: String,
    pub file_size: i64,
}
```

## 12. Error Handling / 错误处理

Use custom error types with proper HTTP status codes:
- `401 Unauthorized` - Invalid credentials or expired token
- `403 Forbidden` - Insufficient subscription tier
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Database or file system errors

## 13. Development Notes / 开发注意事项

- Follow coding specifications in `llm_readme.md`
- Use Chinese for internal comments
- Dual-language documentation for public APIs
- Update `llm_log.txt` after each feature
- Keep implementation minimal and focused
