pub mod migration;
pub mod schema;

use crate::error::{AppError, Result};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;
use std::str::FromStr;

/// Initializes the SQLite database connection pool and runs schema migrations.
///
/// # Arguments
/// * `db_path` - The file path to the SQLite database.
/// * `require_existing` - If true, an error is returned if the database file does not already exist.
///
/// # Returns
/// Returns a `SqlitePool` upon successful initialization.
///
/// # Errors
/// Returns an `AppError::Database` if connection fails, the file doesn't exist when required, or migrations fail.
//
// // 初始化 SQLite 数据库连接池并运行架构迁移。
// //
// // # 参数
// // * `db_path` - SQLite 数据库的文件路径。
// // * `require_existing` - 如果为真，当数据库文件不存在时返回错误。
// //
// // # 返回
// // 成功初始化后返回 `SqlitePool`。
// //
// // # 错误
// // 如果连接失败、要求存在但文件不存在，或迁移失败，则返回 `AppError::Database`。
pub async fn init_db(db_path: &str, require_existing: bool) -> Result<SqlitePool> {
    let db_path_obj = Path::new(db_path);

    // 1. 检查数据库文件是否必须存在。
    if require_existing && !db_path_obj.exists() {
        return Err(AppError::Database(format!(
            "Database file not found at '{}' and require-existing-db is set",
            db_path
        )));
    }

    // 2. 如果不强制存在，则尝试创建父目录。
    if !require_existing {
        if let Some(parent) = db_path_obj.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::Database(format!("Failed to create database directory: {}", e))
                })?;
            }
        }
    }

    // 3. 配置连接选项。
    let connection_string = format!("sqlite:{}", db_path);
    let options = SqliteConnectOptions::from_str(&connection_string)
        .map_err(|e| AppError::Database(format!("Invalid database connection string: {}", e)))?
        .create_if_missing(!require_existing);

    // 4. 创建连接池。
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect to database: {}", e)))?;

    // 5. 执行初始化迁移（创建基础表）。
    sqlx::query(schema::SCHEMA)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to execute schema migrations: {}", e)))?;

    // 6. 检查数据库版本并执行版本升级。
    migration::check_and_upgrade(&pool).await?;

    Ok(pool)
}
