// User operations handler
//
// // 用户操作处理器

use super::auth::UserState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::middleware::auth::AuthClaims;
use crate::utils::hash;
use axum::{Json, extract::{Query, State}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Request payload for changing password.
///
/// Requires the current password for verification and the new password.
//
// // 修改密码的请求载荷。
// //
// // 需要当前密码进行验证和新密码。
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password for verification / 用于验证的当前密码
    pub current_password: String,
    /// New password to set / 要设置的新密码
    pub new_password: String,
}

/// Response for change password operation.
///
/// Contains a simple success message.
//
// // 修改密码操作的响应。
// //
// // 包含简单的成功消息。
#[derive(Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

/// Subscription period response without note field.
///
/// Used for user-facing subscription queries where admin notes are hidden.
//
// // 不包含备注字段的订阅时段响应。
// //
// // 用于用户面向的订阅查询，管理员备注被隐藏。
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSubscriptionPeriod {
    pub tier: i32,
    pub start_date: i64,
    pub end_date: i64,
}

/// Change password handler.
///
/// Allows an authenticated user to change their password.
/// Requires verification of the current password before setting the new one.
///
/// # Arguments
/// * `state` - Application state containing database pool
/// * `claims` - Authenticated user claims from JWT
/// * `req` - Request body with current and new password
///
/// # Returns
/// Returns a success message on password change.
///
/// # Errors
/// Returns `Unauthorized` if current password is incorrect.
/// Returns `ValidationError` if new password is empty.
/// Returns `Database` error on database failures.
/// Returns `Internal` error on password hashing failure.
//
// // 修改密码处理器。
// //
// // 允许已认证用户修改密码。
// // 需要先验证当前密码才能设置新密码。
// //
// // # 参数
// // * `state` - 包含数据库连接池的应用状态
// // * `claims` - 来自 JWT 的已认证用户声明
// // * `req` - 包含当前密码和新密码的请求体
// //
// // # 返回
// // 成功修改密码时返回成功消息。
// //
// // # 错误
// // 如果当前密码不正确，返回 `Unauthorized`。
// // 如果新密码为空，返回 `ValidationError`。
// // 数据库失败时返回 `Database` 错误。
// // 密码哈希失败时返回 `Internal` 错误。
pub async fn change_password(
    State(state): State<UserState>,
    claims: AuthClaims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 JWT sub 字段提取用户 hash_id（格式为 "user:<hash_id>"）
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    // 2. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(user_hash_id)?;

    // 3. 验证新密码非空
    if req.new_password.trim().is_empty() {
        return Err(AppError::ValidationError(
            "new_password must not be empty".to_string(),
        ));
    }

    // 4. 从数据库查询当前密码哈希
    let current_hash: Option<(String,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    // 5. 检查用户是否存在
    let current_hash = match current_hash {
        Some(h) => h,
        None => return Err(AppError::NotFound("User not found".to_string())),
    };

    // 6. 验证当前密码
    if !hash::verify_password(&req.current_password, &current_hash.0)? {
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    // 7. 哈希新密码
    let new_hash = hash::hash_password(&req.new_password)?;

    // 8. 更新数据库中的密码
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: ChangePasswordResponse {
            message: "Password changed successfully".to_string(),
        },
    }))
}

/// Get own subscriptions handler.
///
/// Returns a list of subscription periods for the authenticated user.
/// The note field is excluded as it is admin-only information.
///
/// # Arguments
/// * `state` - Application state containing database pool
/// * `claims` - Authenticated user claims from JWT
///
/// # Returns
/// Returns a list of subscription periods with tier, start_date, and end_date.
///
/// # Errors
/// Returns `Internal` error if token subject format is invalid.
/// Returns `Database` error on database failures.
//
// // 获取自己的订阅处理器。
// //
// // 返回已认证用户的订阅时段列表。
// // 备注字段被排除，因为它是仅管理员可见的信息。
// //
// // # 参数
// // * `state` - 包含数据库连接池的应用状态
// // * `claims` - 来自 JWT 的已认证用户声明
// //
// // # 返回
// // 返回包含 tier、start_date 和 end_date 的订阅时段列表。
// //
// // # 错误
// // 如果令牌主题格式无效，返回 `Internal` 错误。
// // 数据库失败时返回 `Database` 错误。
pub async fn get_own_subscriptions(
    State(state): State<UserState>,
    claims: AuthClaims,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 JWT sub 字段提取用户 hash_id（格式为 "user:<hash_id>"）
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    // 2. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(user_hash_id)?;

    // 3. 查询用户的所有订阅（不包含 note 字段）
    let subscriptions = sqlx::query_as::<_, UserSubscriptionPeriod>(
        "SELECT tier, start_date, end_date FROM user_subscriptions WHERE user_id = ? ORDER BY start_date DESC"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: subscriptions,
    }))
}

/// Query parameters for get_own_tier.
//
// // get_own_tier 的查询参数。
#[derive(Deserialize)]
pub struct TierQuery {
    pub at: Option<i64>,
}

/// Response for get_own_tier.
//
// // get_own_tier 的响应。
#[derive(Serialize)]
pub struct TierResponse {
    pub tier: i64,
}

/// Get own subscription tier handler.
///
/// Returns the authenticated user's subscription tier at a specific time.
/// If `at` is omitted, defaults to the current time.
///
/// # Arguments
/// * `state` - Application state containing database pool
/// * `claims` - Authenticated user claims from JWT
/// * `query` - Optional `at` query parameter (Unix timestamp)
///
/// # Returns
/// Returns the user's tier (maximum tier from overlapping subscriptions).
///
/// # Errors
/// Returns `Internal` error if token subject format is invalid.
/// Returns `Database` error on database failures.
//
// // 获取自身订阅等级处理器。
// //
// // 返回已认证用户在特定时间的订阅等级。
// // 如果省略 `at`，则默认为当前时间。
// //
// // # 参数
// // * `state` - 包含数据库连接池的应用状态
// // * `claims` - 来自 JWT 的已认证用户声明
// // * `query` - 可选的 `at` 查询参数（Unix 时间戳）
// //
// // # 返回
// // 返回用户的等级（重叠订阅中的最高等级）。
// //
// // # 错误
// // 如果令牌主题格式无效，返回 `Internal` 错误。
// // 数据库失败时返回 `Database` 错误。
pub async fn get_own_tier(
    State(state): State<UserState>,
    claims: AuthClaims,
    Query(query): Query<TierQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 JWT sub 字段提取用户 hash_id（格式为 "user:<hash_id>"）
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    // 2. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(user_hash_id)?;

    // 3. 确定查询时间点，默认为当前时间
    let at = query.at.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    // 4. 查询该时间点的最高订阅等级
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT MAX(tier) FROM user_subscriptions WHERE user_id = ? AND start_date <= ? AND end_date >= ?",
    )
    .bind(user_id)
    .bind(at)
    .bind(at)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let tier = result.map(|(t,)| t).unwrap_or(0);

    Ok(Json(ApiResponse {
        success: true,
        data: TierResponse { tier },
    }))
}
