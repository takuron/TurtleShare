use super::common::ApiResponse;
use crate::config::AdminConfig;
use crate::error::AppError;
use crate::models::subscription::{
    CreateSubscriptionRequest, SubscriptionResponse, UpdateSubscriptionRequest, UserSubscription,
};
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User, UserResponse};
use crate::utils::{hash, hashid::HashIdManager, jwt::JwtManager, rate_limiter::RateLimiter};
use axum::extract::rejection::JsonRejection;
use axum::{
    Json,
    extract::ConnectInfo,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Admin login request.
//
// // 管理员登录请求。
#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

/// Admin login response.
//
// // 管理员登录响应。
#[derive(Serialize)]
pub struct AdminLoginResponse {
    pub token: String,
}

/// Shared application state for admin routes.
//
// // 管理员路由的共享应用状态。
#[derive(Clone)]
pub struct AdminState {
    pub admin_config: AdminConfig,
    pub jwt_manager: Arc<JwtManager>,
    pub hashid_manager: Arc<HashIdManager>,
    pub rate_limiter: RateLimiter,
    pub pool: SqlitePool,
}

/// Admin login handler.
///
/// Validates credentials against config and returns JWT token.
//
// // 管理员登录处理器。
// //
// // 根据配置验证凭据并返回 JWT 令牌。
pub async fn admin_login(
    State(state): State<AdminState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    payload: Result<Json<AdminLoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 限流检查
    let ip = addr.ip().to_string();
    if !state.rate_limiter.check(&ip).await {
        return Err(AppError::TooManyRequests("Rate limit exceeded".to_string()));
    }

    // 2. 处理 JSON 解析错误
    let Json(req) =
        payload.map_err(|_| AppError::ValidationError("Invalid JSON format".to_string()))?;

    // 3. 验证用户名
    if req.username != state.admin_config.username {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // 4. 验证密码
    if !hash::verify_password(&req.password, &state.admin_config.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // 5. 生成 JWT 令牌（sub 固定为 "admin"）
    let token = state
        .jwt_manager
        .generate_token("admin", &req.username, "admin")
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: AdminLoginResponse { token },
        }),
    ))
}

/// List all users.
///
/// Returns a list of all users.
//
// // 列出所有用户。
// //
// // 返回所有用户的列表。
pub async fn list_users(State(state): State<AdminState>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 转换为带有 hash_id 的响应
    let user_responses: Vec<UserResponse> = users
        .iter()
        .map(|u| u.to_response(state.hashid_manager.encode(u.id).unwrap_or_default()))
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: user_responses,
    }))
}

/// Get user detail.
///
/// Retrieves a single user by hash_id.
//
// // 获取用户详情。
// //
// // 通过 hash_id 检索单个用户。
pub async fn get_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: user.to_response(hash_id),
    }))
}

/// Create user.
///
/// Creates a new user with hashed password.
//
// // 创建用户。
// //
// // 创建带有哈希密码的新用户。
pub async fn create_user(
    State(state): State<AdminState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let password_hash = hash::hash_password(&req.password)?;
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let id = sqlx::query(
        "INSERT INTO users (username, password_hash, email, note, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(&req.email)
    .bind(&req.note)
    .bind(created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            AppError::ValidationError("Username already exists".to_string())
        } else {
            AppError::Database(e.to_string())
        }
    })?
    .last_insert_rowid();

    let user = User {
        id,
        username: req.username,
        password_hash,
        email: req.email,
        note: req.note,
        created_at,
    };

    let hash_id = state.hashid_manager.encode(id)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: user.to_response(hash_id),
        }),
    ))
}

/// Update user.
///
/// Updates an existing user's information.
//
// // 更新用户。
// //
// // 更新现有用户的信息。
pub async fn update_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let mut user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if let Some(username) = req.username {
        user.username = username;
    }
    if let Some(password) = req.password {
        user.password_hash = hash::hash_password(&password)?;
    }
    if let Some(email) = req.email {
        if email.is_empty() {
            user.email = None;
        } else {
            user.email = Some(email);
        }
    }
    if let Some(note) = req.note {
        if note.is_empty() {
            user.note = None;
        } else {
            user.note = Some(note);
        }
    }

    sqlx::query(
        "UPDATE users SET username = ?, password_hash = ?, email = ?, note = ? WHERE id = ?",
    )
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.email)
    .bind(&user.note)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: user.to_response(hash_id),
    }))
}

/// Delete user.
///
/// Removes a user from the database.
//
// // 删除用户。
// //
// // 从数据库中移除用户。
pub async fn delete_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let rows_affected = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "deleted": true }),
    }))
}

/// Query parameters for get_user_tier.
//
// // get_user_tier 的查询参数。
#[derive(Deserialize)]
pub struct TierQuery {
    pub at: Option<i64>,
}

/// Response for get_user_tier.
//
// // get_user_tier 的响应。
#[derive(Serialize)]
pub struct TierResponse {
    pub tier: i64,
}

/// Get user tier.
///
/// Queries a user's subscription tier at a specific time.
//
// // 获取用户等级。
// //
// // 查询用户在特定时间的订阅等级。
pub async fn get_user_tier(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Query(query): Query<TierQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let at = query.at.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT MAX(tier) FROM user_subscriptions WHERE user_id = ? AND start_date <= ? AND end_date >= ?"
    )
    .bind(id)
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

    // 4. 验证等级
    if req.tier < 0 {
        return Err(AppError::ValidationError(
            "tier must be non-negative".to_string(),
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

    // 2. 更新提供的字段
    if let Some(tier) = req.tier {
        if tier < 0 {
            return Err(AppError::ValidationError(
                "tier must be non-negative".to_string(),
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
    // 3. 更新备注字段
    if let Some(note) = req.note {
        if note.is_empty() {
            subscription.note = None;
        } else {
            subscription.note = Some(note);
        }
    }

    // 4. 验证时间范围
    if subscription.start_date > subscription.end_date {
        return Err(AppError::ValidationError(
            "start_date must be before end_date".to_string(),
        ));
    }

    // 5. 更新数据库
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

    // 6. 获取用户的 hash_id 用于响应
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
