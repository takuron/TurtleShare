# TurtleShare

[English](./README.md)

TurtleShare 是一个基于 Rust 的会员内容分发后端，面向单管理员运营场景，适合类似 Patreon / 爱发电 的内容付费与订阅系统。它支持文章发布、附件分发、用户与订阅管理，并可由同一服务提供轻量前端静态资源。

项目基于 Axum、SQLite、sqlx、JWT 和本地文件存储实现，目标是保持部署简单、维护直接。

## 功能特性

- 单管理员后台，使用 JWT 进行鉴权
- 用户账号、密码修改与按时间区间管理的订阅
- 支持 Markdown 内容、附件和 `publish_at` 的文章发布
- 基于订阅等级和发布时间的访问控制
- 使用 UUID 随机路径保存本地上传文件
- 通过 `config.toml` 中的 `[siteinfo]` 向前端透传站点信息
- 启动时自动初始化 SQLite 数据库

## 快速开始

### 1. 环境要求

- 已安装 Rust 工具链

### 2. 配置服务

仓库中已提供示例 [`config.toml`](./config.toml)。当前样例配置里的管理员密码哈希对应本地开发口令 `admin123`，真实部署前应替换。

先生成新的 Argon2id 密码哈希：

```bash
cargo run -- hash-pw your-password
```

然后至少修改 `config.toml` 中这些字段：

```toml
[admin]
username = "admin"
password_hash = "$argon2id$..."

[server]
base_url = "http://127.0.0.1:3000"

[jwt]
base_secret = "change-this-in-production"
```

### 3. 启动服务

```bash
cargo run
```

常用参数：

- `cargo run -- --help`
- `cargo run -- --config path/to/config.toml`
- `cargo run -- --require-existing-db`

首次启动时，TurtleShare 会自动创建 SQLite 数据库文件、初始化 schema，并确保上传目录存在。

### 4. 验证服务是否启动

- 直接访问 `http://127.0.0.1:3000/api/health`
- 或使用任意 HTTP 客户端调用健康检查接口

## 文档

- [`docs/architecture.md`](./docs/architecture.md)
- [`docs/configuration.md`](./docs/configuration.md)
- [`docs/api.md`](./docs/api.md)
- [`docs/database.md`](./docs/database.md)
- [`docs/project-structure.md`](./docs/project-structure.md)
- [`docs/TODO.md`](./docs/TODO.md)

## 许可证

本项目采用 GNU Affero General Public License v3.0，详见 [`LICENSE`](./LICENSE)。
