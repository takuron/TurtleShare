// Public announcement handler - No authentication required
//
// // 公开公告处理器 - 无需身份验证

use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::handlers::public::PublicState;
use crate::models::announcement::AnnouncementData;
use axum::{
    Json,
    extract::State,
    response::IntoResponse,
};

/// Get the current site announcement.
///
/// Reads the announcement from kv_store. Returns null data if no announcement
/// has been published yet.
///
/// # Returns
/// Returns the announcement data (content and updated_at), or null if none exists.
///
/// # Errors
/// Returns `AppError::Database` if the database query fails.
/// Returns `AppError::Internal` if the stored JSON is malformed.
//
// // 获取当前站点公告。
// //
// // 从 kv_store 读取公告。如果尚未发布公告，则返回 null 数据。
// //
// // # 返回
// // 返回公告数据（内容和 updated_at），如果不存在则返回 null。
// //
// // # 错误
// // 如果数据库查询失败，返回 `AppError::Database`。
// // 如果存储的 JSON 格式错误，返回 `AppError::Internal`。
pub async fn get_announcement(
    State(state): State<PublicState>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 kv_store 读取公告
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM kv_store WHERE key = 'announcement'"
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 2. 如果不存在，返回 null
    let data = match row {
        Some((value,)) => {
            let announcement: AnnouncementData = serde_json::from_str(&value)
                .map_err(|e| AppError::Internal(format!("Failed to parse announcement: {}", e)))?;
            serde_json::to_value(&announcement)
                .map_err(|e| AppError::Internal(format!("Failed to serialize announcement: {}", e)))?
        }
        None => serde_json::Value::Null,
    };

    Ok(Json(ApiResponse {
        success: true,
        data,
    }))
}
