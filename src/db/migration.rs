/// Database version tracking and migration module.
///
/// Uses the `kv_store` table to track the current database schema version.
/// On startup, the version is checked and any necessary upgrades are applied.
//
// // 数据库版本跟踪与迁移模块。
// //
// // 使用 `kv_store` 表跟踪当前数据库架构版本。
// // 启动时检查版本并应用必要的升级。
use crate::error::{AppError, Result};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

/// The current database schema version.
/// - Version 1: Original schema without version tracking.
/// - Version 2: Added version tracking in kv_store + new table area.
//
// // 当前数据库架构版本。
// // - 版本 1：原始架构，无版本跟踪。
// // - 版本 2：在 kv_store 中添加版本跟踪 + 新数据表区域。
pub const CURRENT_DB_VERSION: i64 = 2;

/// The key used in kv_store to store the database version.
//
// // kv_store 中用于存储数据库版本的键。
const DB_VERSION_KEY: &str = "db_version";

/// Checks the current database version and applies any pending upgrades.
///
/// This function should be called after the base schema is initialized.
/// - If no version is recorded, the database is assumed to be at version 1
///   (the original schema without version tracking), and it will be upgraded to
///   the current version.
/// - If the version is already current, no action is taken.
/// - If the version is newer than the current code expects, an error is returned.
///
/// # Arguments
/// * `pool` - The database connection pool.
///
/// # Errors
/// Returns an `AppError` if version detection or migration fails.
//
// // 检查当前数据库版本并应用待执行的升级。
// //
// // 此函数应在基础架构初始化后调用。
// // - 如果没有记录版本，则假定数据库为版本 1（原始架构，无版本跟踪），将升级到当前版本。
// // - 如果版本已是当前版本，则不执行任何操作。
// // - 如果版本比当前代码所期望的更新，则返回错误。
// //
// // # 参数
// // * `pool` - 数据库连接池。
// //
// // # 错误
// // 如果版本检测或迁移失败，则返回 `AppError`。
pub async fn check_and_upgrade(pool: &SqlitePool) -> Result<()> {
    // 1. 从 kv_store 读取当前数据库版本。
    let current_version = get_version(pool).await?;

    match current_version {
        None => {
            tracing::info!("No database version found. Assuming version 1 (original schema).");
            // 2. 对无版本的数据库，先记录版本为 1。
            set_version(pool, 1).await?;
            // 3. 执行从版本 1 到当前版本的所有升级。
            upgrade_from(pool, 1).await?;
        }
        Some(v) if v < CURRENT_DB_VERSION => {
            tracing::info!(
                "Database version {} is behind current version {}. Upgrading...",
                v,
                CURRENT_DB_VERSION
            );
            // 4. 从当前记录版本开始逐步升级。
            upgrade_from(pool, v).await?;
        }
        Some(v) if v == CURRENT_DB_VERSION => {
            tracing::info!("Database is at version {} (current). No upgrade needed.", v);
        }
        Some(v) => {
            return Err(AppError::Database(format!(
                "Database version {} is newer than the supported version {}. Please update the application.",
                v, CURRENT_DB_VERSION
            )));
        }
    }

    Ok(())
}

/// Reads the database version from the kv_store.
///
/// # Returns
/// Returns `Some(version)` if a version key exists, or `None` if not found.
//
// // 从 kv_store 读取数据库版本。
// //
// // # 返回
// // 如果版本键存在则返回 `Some(version)`，否则返回 `None`。
async fn get_version(pool: &SqlitePool) -> Result<Option<i64>> {
    let result: Option<(String,)> = sqlx::query_as("SELECT value FROM kv_store WHERE key = ?")
        .bind(DB_VERSION_KEY)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to read database version: {}", e)))?;

    match result {
        Some((v,)) => {
            let version: i64 = v.parse().map_err(|e| {
                AppError::Database(format!("Invalid database version format '{}': {}", v, e))
            })?;
            Ok(Some(version))
        }
        None => Ok(None),
    }
}

/// Writes or updates the database version in the kv_store.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `version` - The version number to record.
//
// // 在 kv_store 中写入或更新数据库版本。
// //
// // # 参数
// // * `pool` - 数据库连接池。
// // * `version` - 要记录的版本号。
async fn set_version(pool: &SqlitePool, version: i64) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "INSERT INTO kv_store (key, value, created_at, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?",
    )
    .bind(DB_VERSION_KEY)
    .bind(version.to_string())
    .bind(now)
    .bind(now)
    .bind(version.to_string())
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to set database version: {}", e)))?;

    Ok(())
}

/// Applies all upgrades starting from the given version up to `CURRENT_DB_VERSION`.
///
/// Each upgrade step is applied sequentially. If any step fails, the process
/// is aborted and an error is returned.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `from_version` - The version to start upgrading from (inclusive).
//
// // 从给定版本开始应用所有升级，直到 `CURRENT_DB_VERSION`。
// //
// // 每个升级步骤按顺序执行。如果任何步骤失败，则中止并返回错误。
// //
// // # 参数
// // * `pool` - 数据库连接池。
// // * `from_version` - 开始升级的版本（含）。
async fn upgrade_from(pool: &SqlitePool, from_version: i64) -> Result<()> {
    let mut current = from_version;

    while current < CURRENT_DB_VERSION {
        let target = current + 1;
        tracing::info!(
            "Upgrading database from version {} to {}...",
            current,
            target
        );

        match target {
            2 => migrate_v1_to_v2(pool).await?,
            // ========================================
            // 在此处添加未来版本的迁移。
            // 例如：
            // 3 => migrate_v2_to_v3(pool).await?,
            // 4 => migrate_v3_to_v4(pool).await?,
            // ========================================
            _ => {
                return Err(AppError::Database(format!(
                    "No migration path from version {} to {}",
                    current, target
                )));
            }
        }

        // 更新 kv_store 中的版本号。
        set_version(pool, target).await?;
        tracing::info!("Database successfully upgraded to version {}.", target);
        current = target;
    }

    Ok(())
}

/// Migration from version 1 to version 2.
///
/// This is the first migration. It adds the database version tracking
/// (already set by the caller) and provides the area for new data tables.
///
/// Add any new table CREATE statements inside this function.
//
// // 从版本 1 迁移到版本 2。
// //
// // 这是第一个迁移。它添加数据库版本跟踪（已由调用者设置），并提供新数据表的区域。
// //
// // 在此函数内添加任何新表的 CREATE 语句。
async fn migrate_v1_to_v2(_pool: &SqlitePool) -> Result<()> {
    // ============================================================
    // 在此处添加版本 2 所需的新数据表。
    // 例如：
    //
    // sqlx::query(r#"
    //     CREATE TABLE IF NOT EXISTS your_new_table (
    //         id INTEGER PRIMARY KEY AUTOINCREMENT,
    //         name TEXT NOT NULL,
    //         created_at INTEGER NOT NULL
    //     );
    // "#)
    // .execute(pool)
    // .await
    // .map_err(|e| AppError::Database(format!("Failed to create your_new_table: {}", e)))?;
    //
    // ============================================================

    tracing::info!(
        "Migration v1->v2: No new tables in this migration. Placeholder ready for future tables."
    );

    Ok(())
}
