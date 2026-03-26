# Project Structure / 项目结构

## Source Code / 源代码

```
src/
├── main.rs              # Entry point with CLI and HTTP server / 入口点（含CLI和HTTP服务器）✅
├── config.rs            # TOML configuration loader / TOML配置加载器 ✅
├── error.rs             # Centralized error types / 集中式错误类型 ✅
├── db/                  # Database layer / 数据库层
│   ├── mod.rs           # Database initialization / 数据库初始化 ✅
│   └── schema.rs        # Database schema / 数据库模式 ✅
├── models/              # Data models / 数据模型
│   ├── mod.rs           # ✅
│   ├── user.rs          # User model / 用户模型 ✅
│   ├── article.rs       # Article model / 文章模型 ✅
│   ├── subscription.rs  # Subscription model / 订阅模型 ✅
│   └── file.rs          # File metadata model / 文件元数据模型 ✅
├── handlers/            # HTTP handlers / HTTP 处理器
│   ├── mod.rs           # ✅
│   ├── admin.rs         # Admin endpoints / 管理员端点 ⏳
│   ├── user.rs          # User endpoints / 用户端点 ⏳
│   ├── article.rs       # Article endpoints / 文章端点 ⏳
│   └── file.rs          # File endpoints / 文件端点 ⏳
├── middleware/          # Middleware / 中间件
│   ├── mod.rs           # ✅
│   └── auth.rs          # Authentication / 鉴权 ⏳
└── utils/               # Utilities / 工具函数
    ├── mod.rs           # ✅
    ├── hash.rs          # Password hashing (Argon2id) / 密码哈希 ✅
    ├── jwt.rs           # JWT utilities / JWT 工具 ⏳
    └── file.rs          # File utilities / 文件工具 ⏳
```

**Legend / 图例:**
- ✅ Implemented / 已实现
- ⏳ Not yet implemented / 暂未实现

## Documentation / 文档

```
docs/
├── architecture.md      # Core architecture overview / 核心架构概述 ✅
├── configuration.md     # Configuration details / 配置详情 ✅
├── api.md              # API endpoints / API 端点 ✅
├── database.md         # Database schema / 数据库模式 ✅
├── project-structure.md # This file / 本文件 ✅
└── TODO.md             # Implementation tasks / 实现任务清单 ✅
```

## Development Tools / 开发工具

```
llm_readme.md           # LLM coding specifications / LLM 编码规范 ✅
llm_log.py              # Changelog maintenance script / 变更日志维护脚本 ✅
Cargo.toml              # Rust project manifest / Rust 项目清单 ✅
```
