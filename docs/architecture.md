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
- **Password Hashing**: Standard Argon2id

## 3. Password Hashing Specification / 密码哈希规范

TurtleShare uses standard **Argon2id** for password hashing. The PHC formatted hash string includes the algorithm, version, parameters, salt, and the hash itself, allowing for consistent verification.

### Parameters / 参数
- **Algorithm**: Argon2id
- **Version**: 0x13 (19)
- **Iterations (t)**: 2
- **Memory (m)**: 19456 KB (19 MB)
- **Parallelism (p)**: 1
- **Hash Length**: 32 bytes

### Server-side logic (Rust):
The server uses the `argon2` crate to hash and verify passwords using the parameters above.

## 4. Error Handling / 错误处理

TurtleShare uses the `thiserror` crate for internal error definitions and `axum`'s `IntoResponse` for consistent API error responses.

### Error Response Format:
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message"
  }
}
```

## 4. Documentation / 文档

- [Configuration / 配置文件](./configuration.md)
- [API Endpoints / API 端点](./api.md)
- [Database Schema / 数据库模式](./database.md)
- [Project Structure / 项目结构](./project-structure.md)
- [TODO / 任务清单](./TODO.md)

## 5. Security Considerations / 安全考虑

- Passwords hashed with Argon2
- JWT tokens with expiration
- File paths randomized (UUID)
- SQL injection prevented by sqlx parameterized queries
- File size limits enforced
- CORS configured appropriately
- Rate limiting recommended for production
