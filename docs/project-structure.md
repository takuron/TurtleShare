# Project Structure / 项目结构

## Source Code / 源代码

```text
src/
├── main.rs                    # Entry point, CLI, and server bootstrap / 入口、CLI 与服务启动
├── config.rs                  # TOML configuration loading / TOML 配置加载
├── error.rs                   # Unified application errors / 统一应用错误
├── db/
│   ├── mod.rs                 # Database initialization / 数据库初始化
│   ├── migration.rs           # Version tracking and migrations / 版本跟踪与迁移
│   └── schema.rs              # SQLite schema / SQLite 模式
├── handlers/
│   ├── mod.rs                 # Handler exports / 处理器导出
│   ├── common.rs              # Shared API response types / 共享 API 响应类型
│   ├── routes.rs              # Main router assembly / 主路由组装
│   ├── static_files.rs        # Static file serving / 静态文件服务
│   ├── admin/
│   │   ├── mod.rs             # Admin module exports / 管理员模块导出
│   │   ├── auth.rs            # Admin login / 管理员登录
│   │   ├── users.rs           # Admin user management / 管理员用户管理
│   │   ├── subscriptions.rs   # Admin subscription management / 管理员订阅管理
│   │   ├── articles.rs        # Admin article management / 管理员文章管理
│   │   ├── files.rs           # Admin file management / 管理员文件管理
│   │   ├── announcement.rs    # Admin announcement management / 管理员公告管理
│   │   └── tier_descriptions.rs # Admin tier description management / 管理员等级说明管理
│   ├── public/
│   │   ├── mod.rs             # Public module exports, shared state, route assembly / 公开模块导出、共享状态、路由组装
│   │   ├── api.rs             # Health check and site info / 健康检查与站点信息
│   │   ├── articles.rs        # Public article access / 公开文章访问
│   │   ├── announcement.rs    # Public announcement access / 公开公告访问
│   │   └── tier_descriptions.rs # Public tier descriptions access / 公开等级说明访问
│   └── user/
│       ├── mod.rs             # User module exports / 用户模块导出
│       ├── auth.rs            # User login / 用户登录
│       ├── operations.rs      # Password and subscriptions / 密码与订阅操作
│       └── articles.rs        # User article access / 用户文章访问
├── middleware/
│   ├── mod.rs                 # Middleware exports / 中间件导出
│   ├── auth.rs                # Admin/user auth middleware / 管理员与用户鉴权中间件
│   ├── cors.rs                # Global CORS enforcement / 全局 CORS 控制
│   └── rate_limiter.rs        # Global rate limit middleware / 全局限流中间件
├── models/
│   ├── mod.rs                 # Model exports / 模型导出
│   ├── user.rs                # User model / 用户模型
│   ├── subscription.rs        # Subscription model / 订阅模型
│   ├── article.rs             # Article model / 文章模型
│   ├── file.rs                # File metadata model / 文件元数据模型
│   ├── announcement.rs        # Announcement model / 公告模型
│   └── tier_description.rs    # Tier description model / 等级说明模型
└── utils/
    ├── mod.rs                 # Utility exports / 工具导出
    ├── hash.rs                # Password hashing / 密码哈希
    ├── hashid.rs              # HashID encoding / HashID 编码
    ├── jwt.rs                 # JWT signing and rotation / JWT 签发与轮换
    ├── rate_limiter.rs        # Sliding-window limiter / 滑动窗口限流器
    └── file.rs                # File helpers / 文件工具
```

## Tests / 测试

```text
tests/
└── system/
    ├── main.rs                # System test entry / 系统测试入口
    ├── common/
    │   └── mod.rs             # Test server bootstrap and helpers / 测试服务器启动与辅助方法
    ├── cors.rs                # CORS integration tests / CORS 集成测试
    ├── health_check.rs        # Health and site-info tests / 健康检查与站点信息测试
    ├── rate_limiter.rs        # Rate limiter tests / 限流测试
    ├── pagination.rs          # Pagination tests / 分页测试
    ├── publish_at.rs          # Publish timestamp tests / 发布时间测试
    ├── public_articles.rs     # Public article tests / 公开文章测试
    ├── admin_auth.rs          # Admin auth tests / 管理员鉴权测试
    ├── admin_users.rs         # Admin user tests / 管理员用户测试
    ├── admin_subscriptions.rs # Admin subscription tests / 管理员订阅测试
    ├── admin_articles.rs      # Admin article tests / 管理员文章测试
    ├── admin_files.rs         # Admin file tests / 管理员文件测试
    ├── user_auth.rs           # User auth tests / 用户鉴权测试
    ├── user_operations.rs     # User operation tests / 用户操作测试
    └── user_articles.rs       # User article tests / 用户文章测试
```

## Documentation / 文档

```text
docs/
├── architecture.md           # Architecture overview / 架构概述
├── api.md                    # API and route behavior / API 与路由行为
├── configuration.md          # Configuration reference / 配置说明
├── database.md               # Database schema / 数据库模式
├── project-structure.md      # This file / 本文件
└── TODO.md                   # Task tracking / 任务追踪
```

## Root Files / 根目录文件

```text
Cargo.toml                    # Rust package manifest / Rust 包清单
Cargo.lock                    # Dependency lockfile / 依赖锁文件
config.toml                   # Runtime configuration / 运行时配置
llm_readme.md                 # LLM development rules / LLM 开发规则
llm_log.py                    # LLM changelog helper / LLM 变更日志脚本
llm_log.txt                   # LLM changelog output / LLM 变更日志输出
README.md                     # English README / 英文 README
README.zh-CN.md               # Chinese README / 中文 README
```
