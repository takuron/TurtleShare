// Admin subscription management handlers
//
// // 管理员订阅管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::subscription::{
    CreateSubscriptionRequest, SubscriptionResponse, UpdateSubscriptionRequest, UserSubscription,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// List user subscriptions.
///
/// Returns all subscriptions for a specific user.
//
// // 列出用户订阅。
// //
// // 返回特定用户的所有订阅。
pub async fn list_user_subscriptions(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(&hash_id)?;

    // 2. 验证用户存在
    let user_exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if user_exists.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // 3. 查询用户的所有订阅
    let subscriptions = sqlx::query_as::<_, UserSubscription>(
        "SELECT id, user_id, tier, start_date, end_date, note, created_at FROM user_subscriptions WHERE user_id = ? ORDER BY start_date DESC"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. 转换为带有 hash_id 的响应
    let responses: Vec<SubscriptionResponse> = subscriptions
        .iter()
        .map(|s| s.to_response(&state.hashid_manager, hash_id.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}

/// Create subscription.
///
/// Adds a new subscription period for a user.
//
// // 创建订阅。
// //
// // 为用户添加新的订阅时段。
pub async fn create_subscription(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(&hash_id)?;

    // 2. 验证用户存在
    let user_exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if user_exists.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // 3. 验证时间范围
    if req.start_date > req.end_date {
        return Err(AppError::ValidationError(
            "start_date must be before end_date".to_string(),
        ));
    }

    // 4. 验证等级范围（0-255）
    if req.tier < 0 || req.tier > 255 {
        return Err(AppError::ValidationError(
            "tier must be between 0 and 255".to_string(),
        ));
    }

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 5. 插入订阅记录
    let id = sqlx::query(
        "INSERT INTO user_subscriptions (user_id, tier, start_date, end_date, note, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(req.tier)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(&req.note)
    .bind(created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .last_insert_rowid();

    let subscription = UserSubscription {
        id,
        user_id,
        tier: req.tier,
        start_date: req.start_date,
        end_date: req.end_date,
        note: req.note,
        created_at,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: subscription.to_response(&state.hashid_manager, hash_id)?,
        }),
    ))
}

/// Update subscription.
///
/// Updates an existing subscription's information.
//
// // 更新订阅。
// //
// // 更新现有订阅的信息。
pub async fn update_subscription(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let subscription_id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询现有订阅
    let mut subscription = sqlx::query_as::<_, UserSubscription>(
        "SELECT id, user_id, tier, start_date, end_date, note, created_at FROM user_subscriptions WHERE id = ?"
    )
    .bind(subscription_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;

    // 3. 更新提供的字段
    if let Some(tier) = req.tier {
        if tier < 0 || tier > 255 {
            return Err(AppError::ValidationError(
                "tier must be between 0 and 255".to_string(),
            ));
        }
        subscription.tier = tier;
    }
    if let Some(start_date) = req.start_date {
        subscription.start_date = start_date;
    }
    if let Some(end_date) = req.end_date {
        subscription.end_date = end_date;
    }
    // 4. 更新备注字段
    if let Some(note) = req.note {
        if note.is_empty() {
            subscription.note = None;
        } else {
            subscription.note = Some(note);
        }
    }

    // 5. 验证时间范围
    if subscription.start_date > subscription.end_date {
        return Err(AppError::ValidationError(
            "start_date must be before end_date".to_string(),
        ));
    }

    // 6. 更新数据库
    sqlx::query(
        "UPDATE user_subscriptions SET tier = ?, start_date = ?, end_date = ?, note = ? WHERE id = ?",
    )
    .bind(subscription.tier)
    .bind(subscription.start_date)
    .bind(subscription.end_date)
    .bind(&subscription.note)
    .bind(subscription_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 7. 获取用户的 hash_id 用于响应
    let user_hash_id = state.hashid_manager.encode(subscription.user_id)?;

    Ok(Json(ApiResponse {
        success: true,
        data: subscription.to_response(&state.hashid_manager, user_hash_id)?,
    }))
}

/// Delete subscription.
///
/// Removes a subscription from the database.
//
// // 删除订阅。
// //
// // 从数据库中移除订阅。
pub async fn delete_subscription(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let subscription_id = state.hashid_manager.decode(&hash_id)?;

    // 2. 先查询订阅以获取 user_id（用于验证存在性）
    let subscription = sqlx::query_as::<_, UserSubscription>(
        "SELECT id, user_id, tier, start_date, end_date, note, created_at FROM user_subscriptions WHERE id = ?"
    )
    .bind(subscription_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;

    // 3. 删除订阅
    sqlx::query("DELETE FROM user_subscriptions WHERE id = ?")
        .bind(subscription_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. 获取用户的 hash_id 用于响应
    let user_hash_id = state.hashid_manager.encode(subscription.user_id)?;

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({
            "deleted": true,
            "hash_id": hash_id,
            "user_hash_id": user_hash_id
        }),
    }))
}
