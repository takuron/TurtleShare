# TurtleShare Architecture Document
# TurtleShare 架构文档

## 1. Overview / 概述

TurtleShare is a single-administrator membership content distribution backend, similar to Afdian or Patreon. It is built with Rust, uses Axum as the HTTP framework, and stores data in SQLite.

TurtleShare 是一个单管理员会员内容分发平台后端，形态类似爱发电或 Patreon。项目使用 Rust 构建，HTTP 框架为 Axum，数据存储为 SQLite。

### Core Features / 核心功能

- Admin authentication / 管理员鉴权
- User authentication and self-service operations / 用户鉴权与自助操作
- Article publishing with Markdown content and file attachments / 支持 Markdown 内容和附件的文章发布
- Subscription tiers with time-based access control / 基于订阅等级和时间区间的访问控制
- Local file storage with UUID-based random paths / 基于 UUID 随机路径的本地文件存储
- Static file serving for frontend assets and uploaded files / 前端资源与上传文件的静态文件服务
- Global rate limiting and global CORS enforcement / 全局限流与全局 CORS 控制

## 2. Technology Stack / 技术栈

- **Web Framework**: Axum
- **Database**: SQLite with sqlx
- **Configuration**: TOML (`config.toml`)
- **Authentication**: JWT tokens
- **File Storage**: Local filesystem with UUID-based paths
- **Password Hashing**: Standard Argon2id

## 3. Password Hashing Specification / 密码哈希规范

TurtleShare uses standard **Argon2id** for password hashing. The PHC formatted hash string contains the algorithm, version, parameters, salt, and hash, so the server can verify passwords consistently.

TurtleShare 使用标准 **Argon2id** 对密码进行哈希。PHC 格式字符串会包含算法、版本、参数、盐值和哈希结果，便于服务端统一完成密码校验。

### Parameters / 参数

- **Algorithm**: Argon2id
- **Version**: 0x13 (19)
- **Iterations (t)**: 2
- **Memory (m)**: 19456 KB (19 MB)
- **Parallelism (p)**: 1
- **Hash Length**: 32 bytes

## 4. Error Handling / 错误处理

TurtleShare uses the `thiserror` crate for internal error types and Axum's `IntoResponse` trait to convert them into a consistent JSON error response.

TurtleShare 使用 `thiserror` 定义内部错误类型，并通过 Axum 的 `IntoResponse` 统一转换为一致的 JSON 错误响应。

### Error Response Format / 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message"
  }
}
```

## 5. Static File Serving / 静态文件服务

TurtleShare serves two categories of static content:

TurtleShare 提供两类静态内容：

1. **Frontend Static Files**: Frontend assets in the configured `static_path` directory. These are used as the SPA fallback for unmatched non-API routes.
1. **前端静态文件**：来自配置项 `static_path` 的前端资源，用作所有未命中的非 API 路由的 SPA 回退。
2. **Uploaded Files**: Files stored under the configured `files_path` directory and exposed through `/files/*`.
2. **上传文件**：保存在配置项 `files_path` 目录下，并通过 `/files/*` 对外提供。

### Route Priority / 路由优先级

- API routes such as `/api/*` are matched first / `/api/*` 等 API 路由优先匹配
- `/files/*` serves uploaded files / `/files/*` 提供上传文件
- Other routes fall back to frontend static files / 其他路由回退到前端静态文件

## 6. CORS Handling / CORS 处理

TurtleShare applies one global CORS middleware to all HTTP routes, including API routes, uploaded files, and frontend static file responses.

TurtleShare 对所有 HTTP 路由统一应用一层全局 CORS 中间件，包括 API 路由、上传文件和前端静态文件响应。

- Same-origin requests derived from `server.base_url` are always allowed / 基于 `server.base_url` 推导出的同源请求始终允许
- Additional allowed origins come from `server.cors_origins` / 额外允许的来源来自 `server.cors_origins`
- Requests from other origins are rejected with `403 Forbidden` / 其他来源的请求会被直接拒绝并返回 `403 Forbidden`
- Valid preflight requests are handled before authentication middleware / 合法预检请求会在鉴权中间件之前处理

## 7. Documentation / 文档

- [Configuration / 配置文件](./configuration.md)
- [API Endpoints / API 端点](./api.md)
- [Database Schema / 数据库模式](./database.md)
- [Project Structure / 项目结构](./project-structure.md)
- [TODO / 任务清单](./TODO.md)

## 8. Security Considerations / 安全考虑

- Passwords are hashed with Argon2 / 密码使用 Argon2 哈希
- JWT tokens have expiration and rotation support / JWT 令牌具有过期和轮换机制
- File paths are randomized with UUIDs / 文件路径通过 UUID 随机化
- SQL injection is mitigated by sqlx parameterized queries / 使用 sqlx 参数化查询降低 SQL 注入风险
- File size limits are enforced on uploads / 上传文件大小受到限制
- CORS is enforced globally with same-origin and whitelist rules / 全局 CORS 采用“同源 + 白名单”规则
- API routes are protected by rate limiting / API 路由受限流保护
