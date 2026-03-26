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
