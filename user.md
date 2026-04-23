# TurtleShare 用户手册

## 目录

1. [项目简介](#项目简介)
2. [技术架构](#技术架构)
3. [快速开始](#快速开始)
4. [后端配置说明](#后端配置说明)
5. [前端开发指南](#前端开发指南)
6. [API 接口概述](#api-接口概述)
7. [常见问题](#常见问题)

---

## 项目简介

### 什么是 TurtleShare？

**TurtleShare** 是一个面向单管理员运营场景的会员内容分发平台，功能类似于 **爱发电 (Afdian)** 或 **Patreon**。它允许创作者：

- 发布带有 Markdown 内容的文章
- 上传并分发文件附件
- 管理用户账号和订阅
- 基于订阅等级控制内容访问权限

### 核心功能

| 功能模块 | 描述 |
|---------|------|
| **管理员后台** | 单管理员系统，支持用户、文章、文件的全面管理 |
| **用户系统** | 用户注册/登录、密码修改、订阅记录查看 |
| **文章发布** | 支持 Markdown 格式，可设置发布时间和所需订阅等级 |
| **文件管理** | 本地文件上传，使用 UUID 随机路径保护隐私 |
| **订阅系统** | 按时间区间管理的订阅，支持多等级订阅 |
| **访问控制** | 基于订阅等级和发布时间的精细化权限控制 |

### 项目结构

项目分为两个独立的部分：

```
├── TurtleShare/           # 后端项目 (Rust)
│   ├── src/              # 源代码
│   ├── static/           # 静态文件（前端构建产物存放位置）
│   ├── tests/            # 系统测试
│   ├── docs/             # 技术文档
│   ├── config.toml       # 配置文件
│   └── Cargo.toml        # Rust 依赖配置
│
└── turtle-share-svelte/  # 前端项目 (SvelteKit)
    ├── src/              # 源代码
    ├── static/           # 静态资源
    ├── docs/             # 前端文档
    └── package.json      # Node.js 依赖配置
```

---

## 技术架构

### 后端技术栈

| 技术 | 版本 | 用途 |
|-----|------|------|
| Rust | 2024 Edition | 主要开发语言 |
| Axum | 0.8.x | HTTP 框架 |
| SQLite | - | 嵌入式数据库 |
| sqlx | 0.8.x | 数据库 ORM |
| jsonwebtoken | 10.x | JWT 认证 |
| Argon2id | 标准 | 密码哈希 |
| tokio | 1.x | 异步运行时 |

### 前端技术栈

| 技术 | 版本 | 用途 |
|-----|------|------|
| SvelteKit | 2.50+ | 前端框架 |
| Svelte | 5.54+ | 组件库 (Runes 模式) |
| TypeScript | 5.9+ | 类型安全 |
| Tailwind CSS | 4.1 | 样式框架 |
| DaisyUI | 5.5 | UI 组件库 |
| Vite | 7.3 | 构建工具 |
| Paraglide | 2.10+ | 国际化 |

### 架构特点

1. **单二进制部署**：后端编译为单个可执行文件，部署简单
2. **前后端分离**：前端构建为静态文件，由后端统一服务
3. **SPA 路由**：前端使用单页应用路由，后端提供 API 和静态文件服务
4. **JWT 认证**：无状态的 Token 认证机制
5. **SQLite 数据库**：无需额外数据库服务，数据存储在单个文件中

---

## 快速开始

### 环境准备

在开始之前，请确保您的系统已安装以下工具：

#### 必需软件

| 软件 | 版本要求 | 安装方式 |
|-----|---------|---------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| pnpm | 8+ | `npm install -g pnpm` |

#### 验证安装

```bash
# 验证 Rust
rustc --version
cargo --version

# 验证 Node.js
node --version
pnpm --version
```

### 步骤 1：克隆项目

```bash
# 假设项目已在当前目录，或者您需要从 git 克隆
# git clone <repository-url>
cd TurtleShare
```

### 步骤 2：配置后端

#### 2.1 修改配置文件

后端使用 `config.toml` 进行配置。在部署前，建议修改以下关键配置：

```toml
# 管理员账号配置
[admin]
username = "admin"
# 注意：当前密码哈希对应密码 "admin123"，生产环境请替换
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$FVGhfDUHIQpSCUabKbhkVA$e0tpWtkmWL7uKmX2t517HOAHpUuBbmIpluFwDv522Ns"

# 服务器配置
[server]
host = "127.0.0.1"    # 监听地址
port = 3000             # 监听端口
base_url = "http://127.0.0.1:3000"  # 对外访问 URL

# JWT 密钥（生产环境必须修改！）
[jwt]
base_secret = "your-secret-key-change-in-production"
```

#### 2.2 生成新的管理员密码哈希

如果您想修改管理员密码，可以使用以下命令生成新的密码哈希：

```bash
# 在 TurtleShare 目录下执行
cargo run -- hash-pw 您的新密码
```

命令执行后会输出一个 Argon2id 格式的哈希字符串，将其替换到 `config.toml` 的 `password_hash` 字段。

### 步骤 3：安装前端依赖

```bash
# 进入前端目录
cd ../turtle-share-svelte

# 安装依赖
pnpm install
```

### 步骤 4：构建前端

```bash
# 构建生产版本
pnpm build

# 构建完成后，产物位于 build/ 目录
# 需要将这些文件复制到后端的 static/ 目录
```

### 步骤 5：部署前端到后端

将前端构建产物复制到后端的 `static` 目录：

```bash
# Windows 示例
xcopy /E /Y turtle-share-svelte\build\* TurtleShare\static\

# 或手动复制：将 turtle-share-svelte/build/ 下的所有内容
# 复制到 TurtleShare/static/ 目录
```

### 步骤 6：启动后端服务

```bash
# 进入后端目录
cd TurtleShare

# 开发模式启动
cargo run

# 或使用自定义配置文件
cargo run -- --config path/to/config.toml
```

### 步骤 7：验证服务是否正常

#### 健康检查

打开浏览器访问：
```
http://127.0.0.1:3000/api/health
```

如果返回以下 JSON 表示服务正常：
```json
{ "success": true, "data": { "status": "ok" } }
```

#### 访问前端页面

在浏览器中访问：
```
http://127.0.0.1:3000
```

您应该能看到 TurtleShare 的首页。

---

## 后端配置说明

### 完整配置文件详解

`config.toml` 包含以下配置段：

#### 1. 管理员配置 `[admin]`

```toml
[admin]
username = "admin"           # 管理员用户名
password_hash = "..."        # 管理员密码的 Argon2id 哈希
```

#### 2. 服务器配置 `[server]`

```toml
[server]
host = "127.0.0.1"          # 监听地址（0.0.0.0 表示监听所有网卡）
port = 3000                   # 监听端口
base_url = "http://127.0.0.1:3000"  # 对外访问的完整 URL
cors_origins = ["http://localhost:5173/"]  # 允许的 CORS 来源
```

#### 3. 数据库配置 `[database]`

```toml
[database]
path = "./tts_data/database.db"  # SQLite 数据库文件路径
```

#### 4. 存储配置 `[storage]`

```toml
[storage]
static_path = "./static"          # 前端静态文件目录
files_path = "./tts_data/files"   # 上传文件存储目录
max_upload_size_mb = 1024         # 最大上传文件大小（MB）
```

#### 5. JWT 配置 `[jwt]`

```toml
[jwt]
base_secret = "your-secret-key"   # JWT 签名密钥（必须修改！）
expiry_hours = 24                  # Token 有效期（小时）
rotation_days = 30                 # 密钥轮换周期（天）
```

#### 6. HashID 配置 `[hashid]`

```toml
[hashid]
min_length = 6                     # 生成的短 ID 最小长度
```

#### 7. 站点信息配置 `[siteinfo]`

这些信息会展示在前端页面上：

```toml
[siteinfo]
name = "TurtleShare"               # 站点名称
author = "Admin"                   # 作者名称
avatar = ""                         # 头像 URL（可选）
bio = "站点描述"                    # 站点简介

# 社交媒体链接（可配置多个）
[[siteinfo.social_links]]
platform = "github"                 # 平台名称
url = "https://github.com/takuron"  # 链接地址

[[siteinfo.social_links]]
platform = "bilibili"
url = "https://space.bilibili.com/12345"
```

### 支持的社交平台

| platform 值 | 显示图标 |
|------------|---------|
| github | GitHub |
| x / twitter | X (Twitter) |
| bilibili | Bilibili |
| telegram | Telegram |
| email | Email |
| discord | Discord |

---

## 前端开发指南

### 开发模式

在开发前端时，可以使用独立的开发服务器：

```bash
cd turtle-share-svelte
pnpm dev
```

开发服务器默认运行在 `http://localhost:5173`。

### 前端项目结构

```
turtle-share-svelte/
├── src/
│   ├── lib/
│   │   ├── api/           # API 调用封装
│   │   │   ├── admin/     # 管理员 API
│   │   │   ├── public/    # 公开 API
│   │   │   ├── user/      # 用户 API
│   │   │   └── client.ts  # HTTP 客户端
│   │   ├── components/    # 组件库
│   │   │   ├── admin/     # 管理员组件
│   │   │   ├── main/      # 主界面组件
│   │   │   └── shared/    # 共享组件
│   │   ├── config/        # 配置
│   │   ├── i18n/          # 国际化
│   │   ├── stores/        # 状态管理 (Svelte Runes)
│   │   └── utils/         # 工具函数
│   │
│   └── routes/            # 页面路由
│       ├── (auth)/        # 认证相关页面
│       ├── (dashboard)/   # 仪表盘页面
│       └── (main)/        # 主站页面
│
├── static/                 # 静态资源
├── messages/               # 国际化翻译文件
│   ├── en.json            # 英文
│   └── zh-cn.json         # 简体中文
└── package.json
```

### 路由结构

| 路径 | 页面 | 权限要求 |
|-----|------|---------|
| `/` | 首页 | 公开 |
| `/article/[hashid]` | 文章详情 | 公开/订阅 |
| `/subscribe` | 订阅页面 | 公开 |
| `/user` | 用户登录 | 公开 |
| `/dashboard/*` | 用户仪表盘 | 用户登录 |
| `/admin` | 管理员登录 | 公开 |
| `/dashboard/admin/*` | 管理员后台 | 管理员登录 |

### 构建和部署

#### 构建生产版本

```bash
cd turtle-share-svelte
pnpm build
```

构建产物位于 `build/` 目录。

#### 部署到后端

将 `build/` 目录下的所有文件复制到后端的 `static/` 目录：

```bash
# Windows
xcopy /E /Y build\* ..\TurtleShare\static\

# 或手动复制：
# build/_app/ → TurtleShare/static/_app/
# build/*.html → TurtleShare/static/*.html
# 等等...
```

### 开发注意事项

1. **API 基础地址**：开发模式下前端运行在 `:5173`，后端在 `:3000`，需要确保 CORS 配置正确
2. **静态适配器**：前端使用 `adapter-static`，构建为纯静态文件
3. **SPA 路由**：所有非 API 路由都会回退到 `index.html`，由前端处理路由

---

## API 接口概述

### 响应格式

所有 API 响应采用统一格式：

#### 成功响应
```json
{
  "success": true,
  "data": { /* 响应数据 */ }
}
```

#### 错误响应
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述信息"
  }
}
```

### 常见错误码

| 错误码 | HTTP 状态码 | 说明 |
|-------|------------|------|
| `UNAUTHORIZED` | 401 | 未授权或 Token 无效 |
| `FORBIDDEN` | 403 | 权限不足 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `VALIDATION_ERROR` | 400 | 请求参数验证失败 |
| `TOO_MANY_REQUESTS` | 429 | 请求过于频繁（限流） |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

### 接口分类

#### 1. 公开接口（无需认证）

| 方法 | 路径 | 说明 |
|-----|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/public/site-info` | 获取站点信息 |
| GET | `/api/public/articles` | 获取公开文章列表 |
| GET | `/api/public/articles/:hashid` | 获取公开文章详情 |
| GET | `/api/public/announcements` | 获取公告列表 |
| GET | `/api/public/tier-descriptions` | 获取订阅等级描述 |

#### 2. 用户接口（需用户 Token）

| 方法 | 路径 | 说明 |
|-----|------|------|
| POST | `/api/users/login` | 用户登录 |
| PUT | `/api/users/password` | 修改密码 |
| GET | `/api/users/subscriptions` | 获取自己的订阅记录 |
| GET | `/api/users/articles` | 获取可访问的文章列表 |
| GET | `/api/users/articles/:hashid` | 获取文章详情（受订阅等级限制） |

#### 3. 管理员接口（需管理员 Token）

| 方法 | 路径 | 说明 |
|-----|------|------|
| POST | `/api/admin/login` | 管理员登录 |
| GET/POST | `/api/admin/users` | 用户列表/创建用户 |
| GET/PUT/DELETE | `/api/admin/users/:id` | 用户详情/更新/删除 |
| POST | `/api/admin/users/:id/subscriptions` | 为用户添加订阅 |
| GET/POST | `/api/admin/articles` | 文章列表/创建文章 |
| GET/PUT/DELETE | `/api/admin/articles/:id` | 文章详情/更新/删除 |
| GET/POST | `/api/admin/files` | 文件列表/上传文件 |
| GET/DELETE | `/api/admin/files/:id` | 文件详情/删除文件 |
| GET/POST/PUT/DELETE | `/api/admin/announcements` | 公告管理 |
| GET/POST/PUT/DELETE | `/api/admin/tier-descriptions` | 订阅等级管理 |

### 认证方式

API 使用 Bearer Token 认证：

```
Authorization: Bearer <JWT-Token>
```

Token 有效期由 `config.toml` 中的 `jwt.expiry_hours` 配置（默认 24 小时）。

---

## 常见问题

### Q1: 忘记管理员密码怎么办？

**答**：您可以重新生成密码哈希并更新配置文件：

```bash
cd TurtleShare
cargo run -- hash-pw 新密码
```

将输出的哈希值替换到 `config.toml` 的 `admin.password_hash` 字段，然后重启服务。

### Q2: 如何修改站点名称和作者信息？

**答**：编辑 `config.toml` 中的 `[siteinfo]` 段，然后重启后端服务。

### Q3: 上传的文件存储在哪里？

**答**：上传的文件默认存储在 `config.toml` 中 `storage.files_path` 指定的目录（默认为 `./tts_data/files`）。每个文件使用 UUID 随机路径存储。

### Q4: 数据库文件在哪里？如何备份？

**答**：
- 数据库文件位于 `config.toml` 中 `database.path` 指定的路径（默认为 `./tts_data/database.db`）
- 备份只需复制该文件即可
- 建议定期备份 `tts_data/` 整个目录

### Q5: 前端修改后如何更新？

**答**：
1. 在前端目录执行 `pnpm build`
2. 将 `build/` 目录下的所有文件复制到后端的 `static/` 目录
3. 后端无需重启，静态文件会自动更新

### Q6: 如何配置 HTTPS？

**答**：TurtleShare 后端本身不直接支持 HTTPS。建议使用反向代理（如 Nginx、Caddy）来处理 HTTPS：

```nginx
# Nginx 配置示例
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

同时需要修改 `config.toml` 中的 `server.base_url` 为 HTTPS 地址。

### Q7: 如何限制上传文件大小？

**答**：在 `config.toml` 中修改 `storage.max_upload_size_mb` 值（单位为 MB），然后重启服务。

### Q8: 服务启动失败怎么办？

**答**：请检查以下几点：
1. 确认 `config.toml` 配置正确
2. 确认端口未被占用
3. 确认 Rust 工具链已正确安装
4. 查看终端输出的错误信息，根据提示修复

---

## 许可证

本项目采用 **GNU Affero General Public License v3.0** 许可证。详情请参阅 `LICENSE` 文件。

---

## 技术支持

如需更多技术文档，请参阅项目中的以下文件：

- `TurtleShare/docs/` - 后端技术文档
- `turtle-share-svelte/docs/` - 前端技术文档

---

**最后更新**：2026年4月
