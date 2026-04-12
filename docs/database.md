# Database Schema / 数据库模式

**Time Format / 时间格式:** All timestamp fields use Unix timestamps (INTEGER, seconds since epoch) / 所有时间戳字段使用 Unix 时间戳（INTEGER，自纪元以来的秒数）

## Version Tracking / 版本跟踪

The database version is stored in the `kv_store` table using the key `db_version`. On startup, the application checks the version and applies any pending migrations automatically.

数据库版本通过 `kv_store` 表中的 `db_version` 键存储。启动时，应用程序会检查版本并自动应用待执行的迁移。

### Version History / 版本历史

| Version | Description / 描述 |
|---------|---------------------|
| 1 | Original schema without version tracking / 原始架构，无版本跟踪 |
| 2 | Added version tracking in kv_store / 在 kv_store 中添加版本跟踪 |

### Migration Mechanism / 迁移机制

- Databases without a `db_version` key are assumed to be at version 1 and will be automatically upgraded / 没有 `db_version` 键的数据库被假定为版本 1，将自动升级
- Each version step has a dedicated migration function (e.g., `migrate_v1_to_v2`) / 每个版本步骤都有专属的迁移函数（如 `migrate_v1_to_v2`）
- New table creation for future versions should be added in `src/db/migration.rs` / 未来版本的新表创建应添加到 `src/db/migration.rs` 中

## users table / 用户表
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT,
    note TEXT,
    created_at INTEGER NOT NULL
);
```

## user_subscriptions table / 用户订阅记录表
```sql
CREATE TABLE user_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    tier INTEGER NOT NULL,
    start_date INTEGER NOT NULL,
    end_date INTEGER NOT NULL,
    note TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

## articles table / 文章表
```sql
CREATE TABLE articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    cover_image TEXT,
    content TEXT NOT NULL,
    required_tier INTEGER NOT NULL DEFAULT 0,
    is_public INTEGER NOT NULL DEFAULT 0,
    file_links TEXT,
    publish_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## files table / 文件表
```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT UNIQUE NOT NULL,
    original_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
```

## kv_store table / KV存储表
```sql
CREATE TABLE kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### KV Store Keys / KV 存储键

| Key | Description / 描述 |
|-----|---------------------|
| `db_version` | Database schema version number / 数据库架构版本号 |
| `jwt_secret_current` | Current JWT signing secret (base64) / 当前 JWT 签名密钥 |
| `jwt_secret_previous` | Previous JWT signing secret (base64, nullable) / 上一个 JWT 签名密钥 |
| `jwt_secret_date` | Current secret creation time (Unix timestamp as string) / 当前密钥创建时间 |
| `announcement` | Site announcement JSON: `{"content":"...","updated_at":1710928800}` / 站点公告 JSON |
| `tier_descriptions` | Tier descriptions JSON: `{"tiers":[{"tier":1,"name":"Basic","description":"...","price":"¥10/月","purchase_url":"https://..."}],"updated_at":1710928800}` / 等级说明 JSON |
