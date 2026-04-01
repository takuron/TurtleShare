// User authentication handler
//
// // 用户认证处理器

use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::user::LoginRequest;
use crate::utils::{hash, hashid::HashIdManager, jwt::JwtManager, rate_limiter::RateLimiter};
use axum::extract::rejection::JsonRejection;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;

/// Shared application state for user routes.
///
/// Contains dependencies needed for user authentication and operations.
//
// // 用户路由的共享应用状态。
// //
// // 包含用户认证和操作所需的依赖项。
#[derive(Clone)]
pub struct UserState {
    pub jwt_manager: Arc<JwtManager>,
    pub hashid_manager: Arc<HashIdManager>,
    pub rate_limiter: RateLimiter,
    pub pool: SqlitePool,
}

/// User login response.
///
/// Contains the JWT token for authenticated user.
//
// // 用户登录响应。
// //
// // 包含已认证用户的 JWT 令牌。
#[derive(Serialize)]
pub struct UserLoginResponse {
    pub token: String,
}

/// User login handler.
///
/// Validates user credentials against the database and returns a JWT token.
/// The token's `sub` field will be in the format `user:<user_hashid>`.
///
/// # Arguments
/// * `state` - Application state containing JWT manager, HashID manager, and database pool
/// * `addr` - Client's socket address for rate limiting
/// * `payload` - JSON body containing username and password
///
/// # Returns
/// Returns a JWT token on successful authentication.
///
/// # Errors
/// Returns `TooManyRequests` if rate limit is exceeded.
/// Returns `ValidationError` if JSON format is invalid.
/// Returns `Unauthorized` if credentials are invalid.
/// Returns `NotFound` if user does not exist.
/// Returns `Database` error on database failures.
/// Returns `Internal` error on JWT generation failure.
//
// // 用户登录处理器。
// //
// // 根据数据库验证用户凭据并返回 JWT 令牌。
// // 令牌的 `sub` 字段格式为 `user:<用户HashID>`。
// //
// // # 参数
// // * `state` - 包含 JWT 管理器、HashID 管理器和数据库连接池的应用状态
// // * `addr` - 客户端的套接字地址，用于限流
// // * `payload` - 包含用户名和密码的 JSON 请求体
// //
// // # 返回
// // 成功认证时返回 JWT 令牌。
// //
// // # 错误
// // 如果超过限流阈值，返回 `TooManyRequests`。
// // 如果 JSON 格式无效，返回 `ValidationError`。
// // 如果凭据无效，返回 `Unauthorized`。
// // 如果用户不存在，返回 `NotFound`。
// // 数据库失败时返回 `Database` 错误。
// // JWT 生成失败时返回 `Internal` 错误。
pub async fn user_login(
    State(state): State<UserState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 限流检查（5分钟内最多10次请求）
    let ip = addr.ip().to_string();
    if !state.rate_limiter.check(&ip).await {
        return Err(AppError::TooManyRequests("Rate limit exceeded".to_string()));
    }

    // 2. 处理 JSON 解析错误
    let Json(req) =
        payload.map_err(|_| AppError::ValidationError("Invalid JSON format".to_string()))?;

    // 3. 从数据库查询用户
    let user: Option<(i64, String, String)> =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(&req.username)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. 检查用户是否存在
    let (user_id, username, password_hash) = match user {
        Some(u) => u,
        None => return Err(AppError::Unauthorized("Invalid credentials".to_string())),
    };

    // 5. 验证密码
    if !hash::verify_password(&req.password, &password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // 6. 生成用户 HashID
    let user_hash_id = state.hashid_manager.encode(user_id)?;

    // 7. 生成 JWT 令牌（sub 格式为 "user:<user_hashid>"）
    let sub = format!("user:{}", user_hash_id);
    let token = state
        .jwt_manager
        .generate_token(&sub, &username, "user")
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: UserLoginResponse { token },
        }),
    ))
}
