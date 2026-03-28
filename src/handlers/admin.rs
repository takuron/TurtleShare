use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
    extract::ConnectInfo,
};
use axum::extract::rejection::JsonRejection;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::AdminConfig;
use crate::utils::{hash, jwt::JwtManager, rate_limiter::RateLimiter};
use crate::error::AppError;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use super::common::ApiResponse;

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
    let Json(req) = payload.map_err(|_| AppError::ValidationError("Invalid JSON format".to_string()))?;

    // 3. 验证用户名
    if req.username != state.admin_config.username {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // 4. 验证密码
    if !hash::verify_password(&req.password, &state.admin_config.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // 5. 生成 JWT 令牌（sub 固定为 "admin"）
    let token = state.jwt_manager.generate_token("admin", &req.username, "admin").await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: AdminLoginResponse { token },
        })
    ))
}

/// List all users.
///
/// Returns a list of all users.
//
// // 列出所有用户。
// //
// // 返回所有用户的列表。
pub async fn list_users(
    State(state): State<AdminState>,
) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: users,
    }))
}

/// Get user detail.
///
/// Retrieves a single user by ID.
//
// // 获取用户详情。
// //
// // 通过 ID 检索单个用户。
pub async fn get_user(
    State(state): State<AdminState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: user,
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
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

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

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: user,
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
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?"
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
        "UPDATE users SET username = ?, password_hash = ?, email = ?, note = ? WHERE id = ?"
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
        data: user,
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
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
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
    Path(id): Path<i64>,
    Query(query): Query<TierQuery>,
) -> Result<impl IntoResponse, AppError> {
    let at = query.at.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

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
