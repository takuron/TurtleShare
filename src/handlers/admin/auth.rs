// Admin authentication handler and shared state
//
// // 管理员认证处理器和共享状态

use crate::config::AdminConfig;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::utils::{hash, hashid::HashIdManager, jwt::JwtManager, rate_limiter::RateLimiter};
use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::ConnectInfo, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;

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
    pub files_path: String,
    pub max_upload_size_bytes: u64,
    pub base_url: String,
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
