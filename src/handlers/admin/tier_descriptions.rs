// Admin tier description management handler
//
// // 管理员等级说明管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::tier_description::{TierDescription, TierDescriptionsData, UpsertTierDescriptionRequest};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Add or update a tier description.
///
/// Stores tier descriptions as a JSON string in the kv_store table
/// under the key "tier_descriptions". If a description for the same tier
/// already exists, only the provided non-empty fields are overwritten.
/// Tiers are kept sorted by tier level.
///
/// # Arguments
/// * `state` - Shared admin state containing the database pool.
/// * `req` - The upsert request containing the tier description data.
///
/// # Returns
/// Returns the full tier descriptions data on success.
///
/// # Errors
/// Returns `AppError::ValidationError` if all three text fields (name, description, price) are empty or missing.
/// Returns `AppError::Database` if the database operation fails.
//
// // 添加或更新等级说明。
// //
// // 将等级说明以 JSON 字符串的形式存储在 kv_store 表中键为 "tier_descriptions" 的记录中。
// // 如果相同等级的说明已存在，仅覆盖提供的非空字段。等级按等级号排序。
// //
// // # 参数
// // * `state` - 包含数据库连接池的共享管理员状态。
// // * `req` - 包含等级说明数据的请求。
// //
// // # 返回
// // 成功时返回完整的等级说明数据。
// //
// // # 错误
// // 如果三个文本字段（名称、说明、价格）均为空或缺失，返回 `AppError::ValidationError`。
// // 如果数据库操作失败，返回 `AppError::Database`。
pub async fn upsert_tier_description(
    State(state): State<AdminState>,
    Json(req): Json<UpsertTierDescriptionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 提取非空字段值
    let name = req.name.filter(|s| !s.trim().is_empty());
    let description = req.description.filter(|s| !s.trim().is_empty());
    let price = req.price.filter(|s| !s.trim().is_empty());

    // 2. 验证至少有一个文本字段非空
    if name.is_none() && description.is_none() && price.is_none() {
        return Err(AppError::ValidationError(
            "at least one of name, description, or price must not be empty".to_string(),
        ));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 3. 读取现有的等级说明数据
    let mut data = fetch_tier_descriptions(&state.pool)
        .await?
        .unwrap_or(TierDescriptionsData {
            tiers: Vec::new(),
            updated_at: now,
        });

    // 4. 查找已有条目，合并更新
    let existing = data.tiers.iter().position(|t| t.tier == req.tier);

    let entry = if let Some(pos) = existing {
        // 更新模式：仅覆盖提供的非空字段
        let old = &data.tiers[pos];
        TierDescription {
            tier: req.tier,
            name: name.unwrap_or_else(|| old.name.clone()),
            description: description.unwrap_or_else(|| old.description.clone()),
            price: price.unwrap_or_else(|| old.price.clone()),
        }
    } else {
        // 新建模式：未提供的字段默认为空字符串
        TierDescription {
            tier: req.tier,
            name: name.unwrap_or_default(),
            description: description.unwrap_or_default(),
            price: price.unwrap_or_default(),
        }
    };

    if let Some(pos) = existing {
        data.tiers[pos] = entry;
    } else {
        data.tiers.push(entry);
    }

    // 5. 按 tier 排序
    data.tiers.sort_by_key(|t| t.tier);
    data.updated_at = now;

    // 6. 序列化并存入 kv_store
    save_tier_descriptions(&state.pool, &data).await?;

    Ok(Json(ApiResponse {
        success: true,
        data,
    }))
}

/// Delete a tier description.
///
/// Removes the description for the specified tier level from the kv_store.
/// If the tier does not exist, returns a 404 error.
///
/// # Arguments
/// * `state` - Shared admin state containing the database pool.
/// * `tier` - The tier level to delete (0-255).
///
/// # Returns
/// Returns a confirmation with the deleted tier level on success.
///
/// # Errors
/// Returns `AppError::NotFound` if the tier description does not exist.
/// Returns `AppError::Database` if the database operation fails.
//
// // 删除等级说明。
// //
// // 从 kv_store 中移除指定等级的说明。
// // 如果该等级不存在，返回 404 错误。
// //
// // # 参数
// // * `state` - 包含数据库连接池的共享管理员状态。
// // * `tier` - 要删除的等级（0-255）。
// //
// // # 返回
// // 成功时返回删除确认及被删除的等级。
// //
// // # 错误
// // 如果等级说明不存在，返回 `AppError::NotFound`。
// // 如果数据库操作失败，返回 `AppError::Database`。
pub async fn delete_tier_description(
    State(state): State<AdminState>,
    Path(tier): Path<u8>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 读取现有的等级说明数据
    let mut data = match fetch_tier_descriptions(&state.pool).await? {
        Some(d) => d,
        None => {
            return Err(AppError::NotFound(
                "Tier description not found".to_string(),
            ))
        }
    };

    // 2. 查找并删除
    let original_len = data.tiers.len();
    data.tiers.retain(|t| t.tier != tier);
    if data.tiers.len() == original_len {
        return Err(AppError::NotFound(
            "Tier description not found".to_string(),
        ));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    data.updated_at = now;

    // 3. 保存更新后的数据
    save_tier_descriptions(&state.pool, &data).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "deleted": true, "tier": tier }),
    }))
}

/// Reads tier descriptions from kv_store.
///
/// # Errors
/// Returns `AppError::Database` if the database query fails.
/// Returns `AppError::Internal` if the stored JSON is malformed.
//
// // 从 kv_store 读取等级说明。
// //
// // # 错误
// // 如果数据库查询失败，返回 `AppError::Database`。
// // 如果存储的 JSON 格式错误，返回 `AppError::Internal`。
async fn fetch_tier_descriptions(
    pool: &sqlx::SqlitePool,
) -> Result<Option<TierDescriptionsData>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM kv_store WHERE key = 'tier_descriptions'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match row {
        Some((value,)) => {
            let data: TierDescriptionsData = serde_json::from_str(&value).map_err(|e| {
                AppError::Internal(format!("Failed to parse tier_descriptions: {}", e))
            })?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
}

/// Saves tier descriptions to kv_store using UPSERT.
///
/// # Errors
/// Returns `AppError::Internal` if serialization fails.
/// Returns `AppError::Database` if the database operation fails.
//
// // 使用 UPSERT 将等级说明保存到 kv_store。
// //
// // # 错误
// // 如果序列化失败，返回 `AppError::Internal`。
// // 如果数据库操作失败，返回 `AppError::Database`。
async fn save_tier_descriptions(
    pool: &sqlx::SqlitePool,
    data: &TierDescriptionsData,
) -> Result<(), AppError> {
    let value = serde_json::to_string(data)
        .map_err(|e| AppError::Internal(format!("Failed to serialize tier_descriptions: {}", e)))?;

    sqlx::query(
        "INSERT INTO kv_store (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?",
    )
    .bind("tier_descriptions")
    .bind(&value)
    .bind(data.updated_at)
    .bind(data.updated_at)
    .bind(&value)
    .bind(data.updated_at)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}
