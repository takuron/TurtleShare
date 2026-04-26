// Public tier descriptions handler - No authentication required
//
// // 公开等级说明处理器 - 无需身份验证

use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::handlers::public::PublicState;
use crate::models::tier_description::TierDescriptionsData;
use axum::{Json, extract::State, response::IntoResponse};

/// Get all tier descriptions.
///
/// Reads tier descriptions from kv_store. Returns an empty tiers array with
/// `updated_at` set to -1 if no tier descriptions exist.
///
/// # Returns
/// Returns the tier descriptions data (tiers list and updated_at).
/// If none exist, returns `{"tiers": [], "updated_at": -1}`.
///
/// # Errors
/// Returns `AppError::Database` if the database query fails.
/// Returns `AppError::Internal` if the stored JSON is malformed.
//
// // 获取所有等级说明。
// //
// // 从 kv_store 读取等级说明。如果不存在等级说明，
// // 则返回空的 tiers 数组且 updated_at 为 -1。
// //
// // # 返回
// // 返回等级说明数据（等级列表和 updated_at）。
// // 不存在时返回 `{"tiers": [], "updated_at": -1}`。
// //
// // # 错误
// // 如果数据库查询失败，返回 `AppError::Database`。
// // 如果存储的 JSON 格式错误，返回 `AppError::Internal`。
pub async fn get_tier_descriptions(
    State(state): State<PublicState>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 kv_store 读取等级说明
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM kv_store WHERE key = 'tier_descriptions'")
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    // 2. 解析数据；如果不存在，返回空列表和 updated_at = -1
    let td = match row {
        Some((value,)) => serde_json::from_str::<TierDescriptionsData>(&value)
            .map_err(|e| AppError::Internal(format!("Failed to parse tier_descriptions: {}", e)))?,
        None => TierDescriptionsData {
            tiers: Vec::new(),
            updated_at: -1,
        },
    };
    let data = serde_json::to_value(&td)
        .map_err(|e| AppError::Internal(format!("Failed to serialize tier_descriptions: {}", e)))?;

    Ok(Json(ApiResponse {
        success: true,
        data,
    }))
}
