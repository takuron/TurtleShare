# API Endpoints / API 端点

## Admin Endpoints / 管理员端点

### POST /api/admin/login
Admin login endpoint. Validates credentials against config.toml and returns JWT token.

管理员登录端点。根据 config.toml 验证凭据并返回 JWT 令牌。

**Authentication / 鉴权:** None required / 无需鉴权

**Rate Limiting / 限流:** 10 requests per 5 minutes per IP / 每个 IP 每 5 分钟最多 10 次请求

**Request Body / 请求体:**
```json
{
  "username": "admin",
  "password": "your_password"
}
```

**Success Response / 成功响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**JWT Claims / JWT 声明:**
- `sub`: "admin" (fixed for admin) / "admin"（管理员固定值）
- `name`: admin username from config / 来自配置的管理员用户名
- `role`: "admin"
- `exp`: expiration timestamp / 过期时间戳
- `iat`: issued at timestamp / 签发时间戳

**Error Response / 错误响应:** `401 Unauthorized`
```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid credentials"
  }
}
```

**Rate Limit Response / 限流响应:** `429 Too Many Requests`
```json
{
  "success": false,
  "error": {
    "code": "TOO_MANY_REQUESTS",
    "message": "Rate limit exceeded"
  }
}
```

---

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

## User Endpoints / 用户端点
- `POST /api/users/login` - User login / 用户登录
- `PUT /api/users/password` - Change password / 修改密码
- `GET /api/articles` - List accessible articles / 列出可访问文章
- `GET /api/articles/:id` - Get article detail / 获取文章详情

## Public Endpoints / 公开端点

### GET /api
Returns a simple text message indicating the API is running.

返回一个简单的文本消息，表明API正在运行。

**Response / 响应:** Plain text / 纯文本
```
TurtleShare API is running!
```

---

### GET /api/health
Health check endpoint for monitoring service availability.

用于监控服务可用性的健康检查端点。

**Authentication / 鉴权:** None required / 无需鉴权

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "status": "ok"
  }
}
```

---

### GET /api/public/site-info
Returns public site information configured in `config.toml`.

返回在 `config.toml` 中配置的公开站点信息。

**Authentication / 鉴权:** None required / 无需鉴权

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "TurtleShare",
    "author": "Admin",
    "sponsor_link": "https://example.com/sponsor",
    "header_image": "/files/uuid-456/header.jpg",
    "base_url": "https://example.com"
  }
}
```

**Response Fields / 响应字段:**
- `name` (string) - Site name from config / 来自配置的站点名称
- `author` (string) - Site author from config / 来自配置的站点作者
- `sponsor_link` (string|null) - Optional sponsor link / 可选的赞助链接
- `header_image` (string|null) - Optional header image path / 可选的头图路径
- `base_url` (string) - Site base URL / 站点基础URL

---

- `GET /api/public/articles` - List public articles / 列出公开文章
- `GET /api/public/articles/:id` - Get public article detail / 获取公开文章详情

## Static File Routes / 静态文件路由
- `GET /files/*` - Serve uploaded files / 提供上传的文件
- `GET /*` - Serve frontend static files (fallback for SPA) / 提供前端静态文件（SPA回退）

## JSON Response Format / JSON 响应格式

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

## Authentication Flow / 鉴权流程

### Password Handling / 密码处理
TurtleShare uses standard **Argon2id** for password hashing. Passwords should be sent as plain text over HTTPS; the server handles the secure hashing and storage.

**Security Note:** Always transmit passwords over HTTPS.

### Admin Authentication / 管理员鉴权
1. Admin sends credentials to `/api/admin/login`
2. Server validates against config.toml
3. Returns JWT token with `sub: "admin", role: "admin"`
4. Admin includes token in `Authorization: Bearer <token>` header

### User Authentication / 用户鉴权
1. User sends credentials to `/api/users/login`
2. Server validates against database
3. Returns JWT token with `sub: "user:<user_hashid>", role: "user"`
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
