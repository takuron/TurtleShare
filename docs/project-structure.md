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
│   ├── mod.rs           # Module exports / 模块导出 ✅
│   ├── common.rs        # Common types (ApiResponse) / 通用类型 ✅
│   ├── routes.rs        # Main router assembly / 主路由组装器 ✅
│   ├── public.rs        # Public endpoints / 公开端点 ✅
│   ├── static_files.rs  # Static file serving / 静态文件服务 ✅
│   ├── admin.rs         # Admin endpoints (login, user/subscription CRUD with rate limiting) / 管理员端点（登录、用户/订阅CRUD，含限流） ✅
│   ├── user.rs          # User endpoints / 用户端点 ⏳
│   ├── article.rs       # Article endpoints / 文章端点 ⏳
│   └── file.rs          # File endpoints / 文件端点 ⏳
├── middleware/          # Middleware / 中间件
│   ├── mod.rs           # ✅
│   └── auth.rs          # Authentication (admin/user) / 鉴权（管理员/用户） ✅
└── utils/               # Utilities / 工具函数
    ├── mod.rs           # ✅
    ├── hash.rs          # Password hashing (Argon2id) / 密码哈希 ✅
    ├── jwt.rs           # JWT with key rotation / JWT（含密钥轮换） ✅
    ├── rate_limiter.rs  # Sliding window rate limiter / 滑动窗口限流器 ✅
    ├── hashid.rs        # HashID encoding / HashID 编码 ✅
    └── file.rs          # File utilities / 文件工具 ⏳
```

**Legend / 图例:**
- ✅ Implemented / 已实现
- ⏳ Not yet implemented / 暂未实现

## Tests / 测试

```
tests/
└── system/              # System (integration) tests / 系统（集成）测试
    ├── main.rs          # Test harness entry point / 测试入口 ✅
    ├── common/
    │   └── mod.rs       # Shared test utilities (server spawn, auth helpers) / 共享测试工具 ✅
    ├── health_check.rs  # Health check endpoint tests / 健康检查端点测试 ✅
    ├── admin_auth.rs    # Admin authentication tests / 管理员鉴权测试 ✅
    ├── admin_users.rs   # Admin user management tests / 管理员用户管理测试 ✅
    └── admin_subscriptions.rs # Admin subscription CRUD tests / 管理员订阅CRUD测试 ✅
```

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

## Root Files / 根目录文件

```
Cargo.toml              # Rust project manifest / Rust 项目清单 ✅
Cargo.lock              # Dependency lock file / 依赖锁定文件 ✅
config.toml             # Application configuration / 应用配置文件 ✅
LICENSE                 # Project license / 项目许可证 ✅
README.md               # Project README / 项目说明 ✅
llm_readme.md           # LLM coding specifications / LLM 编码规范 ✅
llm_log.py              # Changelog maintenance script / 变更日志维护脚本 ✅
llm_log.txt             # Changelog output / 变更日志输出 ✅
```
