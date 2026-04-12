// Admin announcement management handler
//
// // 管理员公告管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::announcement::{AnnouncementData, PublishAnnouncementRequest};
use axum::{
    Json,
    extract::State,
    response::IntoResponse,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Publish or update the site announcement.
///
/// Stores the announcement content as a JSON string in the kv_store table
/// under the key "announcement". If an announcement already exists, it is
/// overwritten. The `updated_at` timestamp is set automatically.
///
/// If the content is empty or whitespace-only, the announcement is deleted
/// from the kv_store and null data is returned.
///
/// # Arguments
/// * `state` - Shared admin state containing the database pool.
/// * `req` - The publish request containing the announcement content.
///
/// # Returns
/// Returns the stored announcement data on success, or null data if deleted.
///
/// # Errors
/// Returns `AppError::Database` if the database operation fails.
//
// // 发布或更新站点公告。
// //
// // 将公告内容以 JSON 字符串的形式存储在 kv_store 表中键为 "announcement" 的记录中。
// // 如果公告已存在，则覆盖。`updated_at` 时间戳自动设置。
// //
// // 如果内容为空或仅包含空白字符，则从 kv_store 中删除公告，并返回 null 数据。
// //
// // # 参数
// // * `state` - 包含数据库连接池的共享管理员状态。
// // * `req` - 包含公告内容的发布请求。
// //
// // # 返回
// // 成功时返回存储的公告数据，删除时返回 null 数据。
// //
// // # 错误
// // 如果数据库操作失败，返回 `AppError::Database`。
pub async fn publish_announcement(
    State(state): State<AdminState>,
    Json(req): Json<PublishAnnouncementRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 检查内容是否为空，若为空则删除公告
    if req.content.trim().is_empty() {
        sqlx::query("DELETE FROM kv_store WHERE key = 'announcement'")
            .execute(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        return Ok(Json(ApiResponse {
            success: true,
            data: serde_json::Value::Null,
        }));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 2. 构建公告数据
    let announcement = AnnouncementData {
        content: req.content,
        updated_at: now,
    };

    // 3. 序列化为 JSON
    let value = serde_json::to_string(&announcement)
        .map_err(|e| AppError::Internal(format!("Failed to serialize announcement: {}", e)))?;

    // 4. 使用 UPSERT 存入 kv_store
    sqlx::query(
        "INSERT INTO kv_store (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?"
    )
    .bind("announcement")
    .bind(&value)
    .bind(now)
    .bind(now)
    .bind(&value)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let data = serde_json::to_value(&announcement)
        .map_err(|e| AppError::Internal(format!("Failed to serialize announcement: {}", e)))?;

    Ok(Json(ApiResponse {
        success: true,
        data,
    }))
}
