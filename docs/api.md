# API Endpoints / API 端点

## Rate Limiting / 限流

All API endpoints are protected by a global rate limiter.
所有 API 端点均受全局限流器保护。

- **Window**: 1 minute (60 seconds) / 窗口：1 分钟（60 秒）
- **Max Requests**: 100 per IP address per window / 最大请求数：每个 IP 地址每个窗口 100 次
- **Scope**: Applies to all `/api/*` routes (public, user, and admin) / 范围：适用于所有 `/api/*` 路由（公开、用户和管理员）
- **Error Response**: Returns `429 Too Many Requests` with error code `TOO_MANY_REQUESTS` / 错误响应：返回 `429 请求过多`，错误码为 `TOO_MANY_REQUESTS`

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

**Note / 注意:** All user IDs in API responses are hash IDs (encoded strings) for security. Numeric IDs are never exposed. / 所有 API 响应中的用户 ID 都是哈希 ID（编码字符串）以保护安全。数字 ID 永远不会暴露。

### GET /api/admin/users
List all registered users. Password hashes are excluded from the response.

列出所有已注册用户。响应中不包含密码哈希。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "jR3kLm",
      "username": "user1",
      "email": "user1@example.com",
      "note": "A test user",
      "created_at": 1710928800
    }
  ]
}
```

**Note / 注意:** All timestamp fields are Unix timestamps (seconds since epoch) / 所有时间戳字段均为 Unix 时间戳（自纪元以来的秒数）

### GET /api/admin/users/:hash_id
Get detail for a specific user.

获取特定用户的详情。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "jR3kLm",
    "username": "user1",
    "email": "user1@example.com",
    "note": "A test user",
    "created_at": 1710928800
  }
}
```

### GET /api/admin/users/:hash_id/tier?at=<timestamp>
Get a user's subscription tier at a specific time. If `at` is omitted, defaults to the current time.

查询特定时间某个用户的订阅等级。如果省略 `at`，则默认为当前时间。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Query Parameters / 查询参数:**
- `at` (integer, optional) - Unix timestamp (e.g., `1710928800`)

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "tier": 2
  }
}
```

### POST /api/admin/users
Create a new user. 

创建新用户。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Request Body / 请求体:**
```json
{
  "username": "new_user",
  "password": "secure_password",
  "email": "new@example.com",
  "note": "Optional note"
}
```

**Success Response / 成功响应:** `201 Created`
```json
{
  "success": true,
  "data": {
    "hash_id": "pL9mNq",
    "username": "new_user",
    "email": "new@example.com",
    "note": "Optional note",
    "created_at": 1711022400
  }
}
```

### PUT /api/admin/users/:hash_id
Update an existing user. Only the provided fields are updated.

更新现有用户。仅更新提供的字段。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Request Body / 请求体:** (All fields optional / 所有字段可选)
```json
{
  "username": "updated_user",
  "password": "new_secure_password",
  "email": "updated@example.com",
  "note": "Updated note"
}
```

**Success Response / 成功响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "pL9mNq",
    "username": "updated_user",
    "email": "updated@example.com",
    "note": "Updated note",
    "created_at": 1711022400
  }
}
```

### DELETE /api/admin/users/:hash_id
Delete a user and all their associated data (subscriptions).

删除用户及其所有关联数据（订阅）。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

**User Subscriptions / 用户订阅管理**

### GET /api/admin/users/:hash_id/subscriptions
List all subscriptions for a specific user. Subscriptions are ordered by start_date descending.

列出特定用户的所有订阅。订阅按 start_date 降序排列。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "xK9mNq",
      "user_hash_id": "jR3kLm",
      "tier": 2,
      "start_date": 1710928800,
      "end_date": 1713520800,
      "note": "Annual subscription",
      "created_at": 1710928800
    },
    {
      "hash_id": "pL3wRt",
      "user_hash_id": "jR3kLm",
      "tier": 1,
      "start_date": 1709280000,
      "end_date": 1710928800,
      "note": null,
      "created_at": 1709280000
    }
  ]
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "User not found"
  }
}
```

### POST /api/admin/users/:hash_id/subscriptions
Add a new subscription period for a user.

为用户添加新的订阅时段。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - User hash ID / 用户哈希 ID

**Request Body / 请求体:**
```json
{
  "tier": 2,
  "start_date": 1710928800,
  "end_date": 1713520800,
  "note": "Annual subscription"
}
```

**Request Fields / 请求字段:**
- `tier` (integer, required) - Subscription tier level (must be 0-255) / 订阅等级（必须为 0-255）
- `start_date` (integer, required) - Start date as Unix timestamp / 开始日期，Unix 时间戳
- `end_date` (integer, required) - End date as Unix timestamp / 结束日期，Unix 时间戳
- `note` (string|null, optional) - Admin-only note for this subscription / 仅管理员可见的订阅备注

**Validation Rules / 验证规则:**
- `start_date` must be before `end_date` / `start_date` 必须早于 `end_date`
- `tier` must be between 0 and 255 / `tier` 必须在 0 到 255 之间

**Success Response / 成功响应:** `201 Created`
```json
{
  "success": true,
  "data": {
    "hash_id": "nQ7vBx",
    "user_hash_id": "jR3kLm",
    "tier": 2,
    "start_date": 1710928800,
    "end_date": 1713520800,
    "note": "Annual subscription",
    "created_at": 1710928800
  }
}
```

**Error Response / 错误响应:** `400 Bad Request`
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "start_date must be before end_date"
  }
}
```

### PUT /api/admin/subscriptions/:hash_id
Update an existing subscription. Only the provided fields are updated.

更新现有订阅。仅更新提供的字段。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - Subscription hash ID / 订阅哈希 ID

**Request Body / 请求体:** (All fields optional / 所有字段可选)
```json
{
  "tier": 3,
  "start_date": 1710928800,
  "end_date": 1716196800,
  "note": "Upgraded and extended"
}
```

**Validation Rules / 验证规则:**
- After update, `start_date` must be before `end_date` / 更新后 `start_date` 必须早于 `end_date`
- `tier` must be between 0 and 255 if provided / 如果提供，`tier` 必须在 0 到 255 之间

**Success Response / 成功响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "nQ7vBx",
    "user_hash_id": "jR3kLm",
    "tier": 3,
    "start_date": 1710928800,
    "end_date": 1716196800,
    "note": "Upgraded and extended",
    "created_at": 1710928800
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Subscription not found"
  }
}
```

### DELETE /api/admin/subscriptions/:hash_id
Delete a subscription from the database.

从数据库中删除订阅。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - Subscription hash ID / 订阅哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "deleted": true,
    "hash_id": "nQ7vBx",
    "user_hash_id": "jR3kLm"
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Subscription not found"
  }
}
```

**Note / 注意:** All subscription IDs in API requests and responses use hash IDs (encoded strings) for security. Numeric IDs are never exposed. / 所有 API 请求和响应中的订阅 ID 都使用哈希 ID（编码字符串）以保护安全。数字 ID 永远不会暴露。

**Articles / 文章管理**

**Note / 注意:** All article IDs in API requests and responses use hash IDs (encoded strings) for security. Numeric IDs are never exposed. / 所有 API 请求和响应中的文章 ID 都使用哈希 ID（编码字符串）以保护安全。数字 ID 永远不会暴露。

### GET /api/admin/articles
List all articles, ordered by created_at descending.

列出所有文章，按 created_at 降序排列。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "xK9mNq",
      "title": "My Article",
      "cover_image": "/files/uuid-123/cover.jpg",
      "content": "# Hello\nArticle content here.",
      "required_tier": 2,
      "is_public": true,
      "file_links": [{"name": "report.pdf", "url": "https://example.com/files/uuid-123/report.pdf"}],
      "created_at": 1710928800,
      "updated_at": 1710928800
    }
  ]
}
```

### GET /api/admin/articles/:hash_id
Get detail for a specific article.

获取特定文章的详情。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - Article hash ID / 文章哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "xK9mNq",
    "title": "My Article",
    "cover_image": "/files/uuid-123/cover.jpg",
    "content": "# Hello\nArticle content here.",
    "required_tier": 2,
    "is_public": true,
    "file_links": [{"name": "report.pdf", "url": "https://example.com/files/uuid-123/report.pdf"}],
    "created_at": 1710928800,
    "updated_at": 1710928800
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Article not found"
  }
}
```

### POST /api/admin/articles
Create a new article.

创建新文章。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Request Body / 请求体:**
```json
{
  "title": "My Article",
  "cover_image": "/files/uuid-123/cover.jpg",
  "content": "# Hello\nArticle content here.",
  "required_tier": 2,
  "is_public": true,
  "file_links": [{"name": "report.pdf", "url": "https://example.com/files/uuid-123/report.pdf"}]
}
```

**Request Fields / 请求字段:**
- `title` (string, required) - Article title (must not be empty) / 文章标题（不能为空）
- `cover_image` (string|null, optional) - Cover image path / 封面图片路径
- `content` (string, required) - Article content (Markdown) / 文章内容（Markdown）
- `required_tier` (integer, required) - Minimum subscription tier required (must be 0-255) / 访问所需的最低订阅等级（必须为 0-255）
- `is_public` (boolean, required) - Whether the article is publicly listed / 文章是否公开列出
- `file_links` (array, optional, default `[]`) - Array of file link objects, each with `name` (string) and `url` (string, must be an absolute URL starting with `http://` or `https://`) / 文件链接对象数组，每个包含 `name`（字符串）和 `url`（字符串，必须是以 `http://` 或 `https://` 开头的绝对链接），默认为空数组

**Validation Rules / 验证规则:**
- `title` must not be empty or whitespace-only / `title` 不能为空或仅包含空白字符
- `required_tier` must be between 0 and 255 / `required_tier` 必须在 0 到 255 之间

**Success Response / 成功响应:** `201 Created`
```json
{
  "success": true,
  "data": {
    "hash_id": "xK9mNq",
    "title": "My Article",
    "cover_image": "/files/uuid-123/cover.jpg",
    "content": "# Hello\nArticle content here.",
    "required_tier": 2,
    "is_public": true,
    "file_links": [{"name": "report.pdf", "url": "https://example.com/files/uuid-123/report.pdf"}],
    "created_at": 1710928800,
    "updated_at": 1710928800
  }
}
```

**Error Response / 错误响应:** `400 Bad Request`
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "title must not be empty"
  }
}
```

### PUT /api/admin/articles/:hash_id
Update an existing article. Only the provided fields are updated. The `updated_at` timestamp is automatically refreshed.

更新现有文章。仅更新提供的字段。`updated_at` 时间戳会自动刷新。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - Article hash ID / 文章哈希 ID

**Request Body / 请求体:** (All fields optional / 所有字段可选)
```json
{
  "title": "Updated Title",
  "cover_image": "/files/uuid-456/new-cover.jpg",
  "content": "Updated content",
  "required_tier": 3,
  "is_public": false,
  "file_links": [{"name": "new-file.pdf", "url": "https://example.com/files/uuid-789/new-file.pdf"}]
}
```

**Note / 注意:** Setting `cover_image` to an empty string will clear the field (set to null). Setting `file_links` to an empty array `[]` will clear all file links. / 将 `cover_image` 设为空字符串将清除该字段（设为 null）。将 `file_links` 设为空数组 `[]` 将清除所有文件链接。

**Validation Rules / 验证规则:**
- If provided, `title` must not be empty or whitespace-only / 如果提供，`title` 不能为空或仅包含空白字符
- If provided, `required_tier` must be between 0 and 255 / 如果提供，`required_tier` 必须在 0 到 255 之间

**Success Response / 成功响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "xK9mNq",
    "title": "Updated Title",
    "cover_image": "/files/uuid-456/new-cover.jpg",
    "content": "Updated content",
    "required_tier": 3,
    "is_public": false,
    "file_links": [],
    "created_at": 1710928800,
    "updated_at": 1711022400
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Article not found"
  }
}
```

### DELETE /api/admin/articles/:hash_id
Delete an article from the database.

从数据库中删除文章。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - Article hash ID / 文章哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "deleted": true,
    "hash_id": "xK9mNq"
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Article not found"
  }
}
```

**Files / 文件管理**

**Note / 注意:** All file IDs in API requests and responses use hash IDs (encoded strings) for security. Numeric IDs are never exposed. / 所有 API 请求和响应中的文件 ID 都使用哈希 ID（编码字符串）以保护安全。数字 ID 永远不会暴露。

### GET /api/admin/files
List all uploaded files, ordered by created_at descending.

列出所有已上传文件，按 created_at 降序排列。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "xK9mNq",
      "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "original_name": "report.pdf",
      "file_size": 1048576,
      "url": "https://example.com/files/a1b2c3d4-e5f6-7890-abcd-ef1234567890/report.pdf",
      "created_at": 1710928800
    }
  ]
}
```

### GET /api/admin/files/:hash_id
Get metadata for a specific file.

获取特定文件的元数据。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - File hash ID / 文件哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "xK9mNq",
    "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "original_name": "report.pdf",
    "file_size": 1048576,
    "url": "https://example.com/files/a1b2c3d4-e5f6-7890-abcd-ef1234567890/report.pdf",
    "created_at": 1710928800
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "File not found"
  }
}
```

### POST /api/admin/files
Upload a file via multipart form data. The file is stored under a UUID v4 directory with its original filename.

通过 multipart 表单数据上传文件。文件存储在 UUID v4 目录下，保留原始文件名。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Request Body / 请求体:** `multipart/form-data`
- `file` (file, required) - The file to upload / 要上传的文件

**Validation Rules / 验证规则:**
- File size must not exceed `max_upload_size_mb` from config (default: 1024 MB) / 文件大小不得超过配置中的 `max_upload_size_mb`（默认：1024 MB）
- A `file` field must be present in the multipart form / multipart 表单中必须包含 `file` 字段

**Success Response / 成功响应:** `201 Created`
```json
{
  "success": true,
  "data": {
    "hash_id": "pL9mNq",
    "uuid": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
    "original_name": "image.png",
    "file_size": 204800,
    "url": "https://example.com/files/b2c3d4e5-f6a7-8901-bcde-f12345678901/image.png",
    "created_at": 1711022400
  }
}
```

**Error Response / 错误响应:** `400 Bad Request`
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "File size exceeds maximum allowed size of 1024 MB"
  }
}
```

### DELETE /api/admin/files/:hash_id
Delete a file from the database and disk.

从数据库和磁盘中删除文件。

**Authentication / 鉴权:** Admin JWT / 管理员 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string) - File hash ID / 文件哈希 ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "deleted": true,
    "hash_id": "xK9mNq"
  }
}
```

**Error Response / 错误响应:** `404 Not Found`
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "File not found"
  }
}
```

## User Endpoints / 用户端点

### POST /api/users/login
User login endpoint. Validates credentials against the database and returns a JWT token.

用户登录端点。根据数据库验证凭据并返回 JWT 令牌。

**Authentication / 鉴权:** None required / 无需鉴权

**Request Body / 请求体:**
```json
{
  "username": "user1",
  "password": "user_password"
}
```

**Request Fields / 请求字段:**
- `username` (string, required) - Username / 用户名
- `password` (string, required) - Password / 密码

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

**Response Fields / 响应字段:**
- `token` (string) - JWT token for authenticated user / 已认证用户的 JWT 令牌

**Error Responses / 错误响应:**

`400 Bad Request` - Invalid JSON format / JSON 格式无效
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid JSON format"
  }
}
```

`401 Unauthorized` - Invalid credentials / 凭据无效
```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid credentials"
  }
}
```

`429 Too Many Requests` - Rate limit exceeded / 超过限流阈值
```json
{
  "success": false,
  "error": {
    "code": "TOO_MANY_REQUESTS",
    "message": "Rate limit exceeded"
  }
}
```

**Notes / 注意事项:**
- Rate limit: 10 requests per 5 minutes per IP / 限流：每个 IP 每 5 分钟最多 10 次请求
- The JWT token's `sub` field is in format `user:<user_hashid>` / JWT 令牌的 `sub` 字段格式为 `user:<用户HashID>`
- Token should be included in `Authorization: Bearer <token>` header for protected endpoints / 令牌应在受保护端点的 `Authorization: Bearer <token>` 头中包含

---

### PUT /api/users/password
Change password endpoint. Allows an authenticated user to change their password.

修改密码端点。允许已认证用户修改密码。

**Authentication / 鉴权:** User JWT required / 需要用户 JWT

**Request Body / 请求体:**
```json
{
  "current_password": "old_password",
  "new_password": "new_secure_password"
}
```

**Request Fields / 请求字段:**
- `current_password` (string, required) - Current password for verification / 用于验证的当前密码
- `new_password` (string, required) - New password to set / 要设置的新密码

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "message": "Password changed successfully"
  }
}
```

**Error Responses / 错误响应:**

`400 Bad Request` - New password is empty / 新密码为空
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "new_password must not be empty"
  }
}
```

`401 Unauthorized` - Current password is incorrect / 当前密码不正确
```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Current password is incorrect"
  }
}
```

---

### GET /api/users/subscriptions
Get own subscriptions endpoint. Returns a list of subscription periods for the authenticated user. The note field is excluded as it is admin-only information.

获取自己的订阅端点。返回已认证用户的订阅时段列表。备注字段被排除，因为它是仅管理员可见的信息。

**Authentication / 鉴权:** User JWT required / 需要用户 JWT

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "tier": 2,
      "start_date": 1700000000,
      "end_date": 1731542400
    },
    {
      "tier": 1,
      "start_date": 1690000000,
      "end_date": 1699999999
    }
  ]
}
```

**Response Fields / 响应字段:**
- `tier` (integer) - Subscription tier level / 订阅等级
- `start_date` (integer) - Start date as Unix timestamp / 开始日期，Unix 时间戳
- `end_date` (integer) - End date as Unix timestamp / 结束日期，Unix 时间戳

**Notes / 注意事项:**
- Results are ordered by `start_date` descending / 结果按 `start_date` 降序排列
- The `note` field is intentionally excluded for user privacy / `note` 字段为保护用户隐私而被排除

---

### GET /api/users/articles
List visible articles endpoint. Returns articles visible to the user based on their subscription tier at article publish time.

列出可见文章端点。返回用户根据文章发布时的订阅等级可见的文章。

**Authentication / 鉴权:** User JWT required / 需要用户 JWT

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "abc123",
      "title": "Premium Article",
      "cover_image": "/files/uuid/cover.jpg",
      "required_tier": 2,
      "accessible": true,
      "created_at": 1700000000,
      "updated_at": 1700000000
    },
    {
      "hash_id": "def456",
      "title": "Public Article",
      "cover_image": null,
      "required_tier": 1,
      "accessible": false,
      "created_at": 1690000000,
      "updated_at": 1690000000
    }
  ]
}
```

**Response Fields / 响应字段:**
- `hash_id` (string) - Article hash ID / 文章 Hash ID
- `title` (string) - Article title / 文章标题
- `cover_image` (string|null) - Cover image path / 封面图片路径
- `required_tier` (integer) - Minimum tier required to fully access / 完整访问所需的最低等级
- `accessible` (boolean) - Whether user can fully access the article content / 用户是否可以完整访问文章内容
- `created_at` (integer) - Creation timestamp / 创建时间戳
- `updated_at` (integer) - Last update timestamp / 最后更新时间戳

**Notes / 注意事项:**
- An article is visible if user had sufficient tier at publish time (`accessible = true`) OR article is public (`accessible = false`) / 文章在用户发布时有足够等级（`accessible = true`）或文章是公开的（`accessible = false`）时可见
- The `content`, `is_public`, and `file_links` fields are intentionally excluded / `content`、`is_public` 和 `file_links` 字段被有意排除
- Access is determined by user's tier at article's `created_at` time, not current tier / 访问权限由用户在文章 `created_at` 时间的等级决定，而非当前等级
- Results are ordered by `created_at` descending / 结果按 `created_at` 降序排列

---

### GET /api/users/articles/:hash_id
Get article detail endpoint with time-based access control. Returns full article content only if user had sufficient tier at article publish time.

获取文章详情端点，带基于时间的访问控制。仅当用户在文章发布时有足够等级时返回完整文章内容。

**Authentication / 鉴权:** User JWT required / 需要用户 JWT

**Path Parameters / 路径参数:**
- `hash_id` (string, required) - Article hash ID / 文章 Hash ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "abc123",
    "title": "Premium Article",
    "cover_image": "/files/uuid/cover.jpg",
    "content": "Premium content here...",
    "required_tier": 2,
    "is_public": false,
    "file_links": [],
    "created_at": 1700000000,
    "updated_at": 1700000000
  }
}
```

**Error Responses / 错误响应:**

`404 Not Found` - Article does not exist / 文章不存在
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Article not found"
  }
}
```

`403 Forbidden` - Insufficient subscription tier at article publish time / 文章发布时订阅等级不足
```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient subscription tier to access this article"
  }
}
```

**Notes / 注意事项:**
- Access is determined solely by user's tier at article's `created_at` time / 访问权限仅由用户在文章 `created_at` 时间的等级决定
- Public status (`is_public`) does NOT grant access to full article content / 公开状态（`is_public`）不授予完整文章内容的访问权限
- Even if an article appears in the list with `accessible: false`, requesting its detail will return 403 / 即使文章在列表中显示为 `accessible: false`，请求其详情将返回 403

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
Returns all key-value pairs from the `[siteinfo]` section of `config.toml` as a JSON object.
The response shape is fully controlled by the configuration file — any keys added to `[siteinfo]` appear in the response automatically.

返回 `config.toml` 中 `[siteinfo]` 部分的所有键值对（序列化为 JSON 对象）。
响应结构完全由配置文件决定——任何添加到 `[siteinfo]` 的键都会自动出现在响应中。

**Authentication / 鉴权:** None required / 无需鉴权

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "name": "TurtleShare",
    "author": "Admin",
    "sponsor_link": "",
    "header_image": "",
    "theme_color": "#3498db"
  }
}
```

**Response Fields / 响应字段:**
The `data` object mirrors `[siteinfo]` in `config.toml` exactly. All TOML scalar types (string, integer, float, boolean), arrays, and inline tables are supported.

`data` 对象与 `config.toml` 中的 `[siteinfo]` 完全对应。支持所有 TOML 标量类型（字符串、整数、浮点数、布尔值）、数组和内联表。

---

### GET /api/public/articles
List public articles endpoint. Returns articles that are publicly listed (is_public = true).
The accessible field indicates if the content can be fully accessed by unauthenticated users.

列出公开文章端点。返回公开列出的文章（is_public = true）。
accessible 字段指示未认证用户是否可以完整访问内容。

**Authentication / 鉴权:** None required / 无需鉴权

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "hash_id": "abc123",
      "title": "Free Article",
      "cover_image": "/files/uuid/cover.jpg",
      "required_tier": 0,
      "accessible": true,
      "created_at": 1700000000,
      "updated_at": 1700000000
    },
    {
      "hash_id": "def456",
      "title": "Premium Preview",
      "cover_image": null,
      "required_tier": 2,
      "accessible": false,
      "created_at": 1690000000,
      "updated_at": 1690000000
    }
  ]
}
```

**Response Fields / 响应字段:**
- `hash_id` (string) - Article hash ID / 文章 Hash ID
- `title` (string) - Article title / 文章标题
- `cover_image` (string|null) - Cover image path / 封面图片路径
- `required_tier` (integer) - Minimum tier required to fully access / 完整访问所需的最低等级
- `accessible` (boolean) - Whether unauthenticated users can fully access the article content / 未认证用户是否可以完整访问文章内容
- `created_at` (integer) - Creation timestamp / 创建时间戳
- `updated_at` (integer) - Last update timestamp / 最后更新时间戳

**Notes / 注意事项:**
- The `content`, `is_public`, and `file_links` fields are intentionally excluded / `content`、`is_public` 和 `file_links` 字段被有意排除
- Results are ordered by `created_at` descending / 结果按 `created_at` 降序排列
- `accessible = true` when `required_tier = 0`, meaning full content is available via detail endpoint / 当 `required_tier = 0` 时 `accessible = true`，表示可通过详情端点获取完整内容
- `accessible = false` when `required_tier > 0`, meaning detail endpoint will return 403 Forbidden / 当 `required_tier > 0` 时 `accessible = false`，表示详情端点将返回 403 Forbidden

---

### GET /api/public/articles/:hash_id
Get public article detail endpoint. Returns full article content only if required_tier = 0.
Returns 403 Forbidden if required_tier > 0.

获取公开文章详情端点。仅当 required_tier = 0 时返回完整文章内容。
如果 required_tier > 0，返回 403 Forbidden。

**Authentication / 鉴权:** None required / 无需鉴权

**Path Parameters / 路径参数:**
- `hash_id` (string, required) - Article hash ID / 文章 Hash ID

**Response / 响应:** `200 OK`
```json
{
  "success": true,
  "data": {
    "hash_id": "abc123",
    "title": "Free Article",
    "cover_image": "/files/uuid/cover.jpg",
    "content": "Full article content here...",
    "required_tier": 0,
    "is_public": true,
    "file_links": [
      {
        "name": "document.pdf",
        "url": "https://example.com/files/uuid/document.pdf"
      }
    ],
    "created_at": 1700000000,
    "updated_at": 1700000000
  }
}
```

**Response Fields / 响应字段:**
- `hash_id` (string) - Article hash ID / 文章 Hash ID
- `title` (string) - Article title / 文章标题
- `cover_image` (string|null) - Cover image path / 封面图片路径
- `content` (string) - Article content (Markdown) / 文章内容（Markdown）
- `required_tier` (integer) - Minimum tier required to access / 访问所需的最低等级
- `is_public` (boolean) - Whether article is publicly listed / 文章是否公开列出
- `file_links` (array) - File links with name and url / 包含名称和链接的文件链接数组
- `created_at` (integer) - Creation timestamp / 创建时间戳
- `updated_at` (integer) - Last update timestamp / 最后更新时间戳

**Error Responses / 错误响应:**

`404 Not Found` - Article does not exist or is not public (is_public = false) / 文章不存在或不是公开的（is_public = false）
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Article not found"
  }
}
```

`403 Forbidden` - Article requires subscription (required_tier > 0) / 文章需要订阅（required_tier > 0）
```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient subscription tier to access this article"
  }
}
```

**Notes / 注意事项:**
- Only public articles (is_public = true) can be accessed via this endpoint / 只有公开文章（is_public = true）可通过此端点访问
- Full content is only returned when required_tier = 0 / 仅当 required_tier = 0 时返回完整内容
- Articles with required_tier > 0 return 403 Forbidden even if is_public = true / 即使 is_public = true，required_tier > 0 的文章返回 403 Forbidden
- Non-public articles (is_public = false) return 404 / 非公开文章（is_public = false）返回 404

---

## Static File Routes / 静态文件路由
- `GET /files/*` - Serve uploaded files without authentication / 无需认证提供上传的文件
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
- `jwt_secret_date` - Current secret creation time (Unix timestamp) / 当前密钥创建时间

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
