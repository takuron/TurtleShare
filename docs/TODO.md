# TODO - Implementation Tasks / 实现任务清单

## Phase 1: Foundation / 基础阶段

### Configuration / 配置
- [x] Implement config.rs to load TOML configuration
- [x] Add ServerConfig with base_url field
- [x] Add StorageConfig with max_upload_size_mb field
- [x] Add SiteInfoConfig struct (name, author, sponsor_link, header_image)
- [x] Validate configuration on startup

### Database / 数据库
- [x] Create database initialization module
- [x] Implement schema.sql with all tables (users, user_subscriptions, articles, files, kv_store)
- [x] Add database migration support
- [x] Create connection pool setup

### JWT Utilities / JWT 工具
- [x] Implement JWT secret generation (SHA256 + UUID)
- [x] Add JWT secret rotation logic
- [x] Implement token signing with jwt_secret_current
- [x] Implement token verification (try current, fallback to previous)
- [x] Add KV store operations for JWT secrets

## Phase 2: Admin Features / 管理员功能

### Admin Authentication / 管理员认证
- [x] Implement POST /api/admin/login endpoint
- [x] Validate credentials against config.toml
- [x] Return JWT token with role: admin
- [x] Add admin authentication middleware

### User Management / 用户管理
- [x] Implement GET /api/admin/users (list all users)
- [x] Implement GET /api/admin/users/:id (get user detail)
- [x] Implement GET /api/admin/users/:id/tier?at=<timestamp> (query tier at time)
- [x] Implement POST /api/admin/users (create user)
- [x] Implement PUT /api/admin/users/:id (update user)
- [x] Implement DELETE /api/admin/users/:id (delete user)

### User Subscriptions / 用户订阅管理
- [x] Implement GET /api/admin/users/:id/subscriptions (list user subscriptions)
- [x] Implement POST /api/admin/users/:id/subscriptions (add subscription period)
- [x] Implement PUT /api/admin/subscriptions/:id (update subscription)
- [x] Implement DELETE /api/admin/subscriptions/:id (delete subscription)

### Article Management / 文章管理
- [x] Implement GET /api/admin/articles (list all articles)
- [x] Implement GET /api/admin/articles/:id (get article detail)
- [x] Implement POST /api/admin/articles (create article)
- [x] Implement PUT /api/admin/articles/:id (update article)
- [x] Implement DELETE /api/admin/articles/:id (delete article)

### File Management / 文件管理
- [x] Implement GET /api/admin/files (list all files)
- [x] Implement GET /api/admin/files/:id (get file metadata)
- [x] Implement POST /api/admin/files (upload file with UUID path)
- [x] Implement DELETE /api/admin/files/:id (delete file)
- [x] Add file size validation (max_upload_size_mb)
- [x] Generate UUID v4 for file directories

## Phase 3: User Features / 用户功能

### User Authentication / 用户认证
- [x] Implement POST /api/users/login endpoint
- [x] Validate credentials against database
- [x] Return JWT token with role: user, user_id
- [x] Add user authentication middleware

### User Operations / 用户操作
- [x] Implement PUT /api/users/password (change password)
- [x] Implement GET /api/users/subscriptions (view own subscription periods without note)
- [x] Implement GET /api/articles (list accessible articles based on tier)
- [x] Implement GET /api/articles/:id (get article detail with access control)
- [x] Add tier-based access control logic

### Public Endpoints / 公开端点
- [ ] Implement GET /api/public/articles (list public articles with title+cover+tier)
- [ ] Implement GET /api/public/articles/:id (get public article, show content if tier=0)
- [x] Implement GET /api/public/site-info (return site configuration)
- [ ] Implement GET /files/:uuid/:filename (serve files without auth)

## Phase 4: Polish / 完善

### Error Handling / 错误处理
- [ ] Implement unified error response format
- [ ] Add error codes (UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, INTERNAL_ERROR)
- [ ] Add proper HTTP status codes
- [ ] Handle database errors gracefully
- [ ] Handle file system errors gracefully

### Input Validation / 输入验证
- [ ] Validate all request parameters
- [ ] Sanitize user inputs
- [ ] Validate file uploads (size, type)
- [x] Migrate time format from RFC 3339 strings to Unix timestamps

### Security / 安全
- [ ] Implement Argon2 password hashing
- [ ] Add CORS configuration
- [ ] Prevent SQL injection (use parameterized queries)
- [ ] Prevent directory traversal in file paths
- [ ] Add rate limiting (recommended for production)

### Logging / 日志
- [x] Add structured logging
- [ ] Log authentication attempts
- [ ] Log file operations
- [ ] Log errors with context

## Additional Features / 额外功能

- [ ] Add pagination for list endpoints
- [ ] Add search/filter for articles
- [ ] Add article sorting options
- [ ] Optimize database queries with indexes
- [x] Add health check endpoint
