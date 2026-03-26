# Project Structure / 项目结构

## Source Code / 源代码

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

## Documentation / 文档

```
docs/
├── architecture.md      # Core architecture overview / 核心架构概述
├── configuration.md     # Configuration details / 配置详情
├── api.md              # API endpoints / API 端点
├── database.md         # Database schema / 数据库模式
├── project-structure.md # This file / 本文件
└── TODO.md             # Implementation tasks / 实现任务清单
```

## Data Directory / 数据目录

```
data/
├── turtleshare.db      # SQLite database / SQLite 数据库
└── files/              # File storage / 文件存储
    └── {uuid}/         # UUID-based directories / 基于UUID的目录
        └── {filename}  # Original filename / 原始文件名
```

## Configuration / 配置

```
config.toml             # Main configuration file / 主配置文件
```
