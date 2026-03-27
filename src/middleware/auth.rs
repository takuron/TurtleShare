use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use crate::utils::jwt::{JwtManager, Claims};
use crate::error::AppError;

/// Extension type to store authenticated claims in request.
//
// // 扩展类型，用于在请求中存储已认证的声明。
#[derive(Clone)]
pub struct AuthClaims(pub Claims);

/// Admin authentication middleware.
///
/// Verifies JWT token and ensures role is "admin".
//
// // 管理员身份验证中间件。
// //
// // 验证 JWT 令牌并确保角色为 "admin"。
pub async fn require_admin(
    State(jwt_manager): State<Arc<JwtManager>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 Authorization 头
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    // 2. 提取 Bearer 令牌
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

    // 3. 验证令牌
    let claims = jwt_manager.verify_token(token).await?;

    // 4. 检查角色
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // 5. 将声明添加到请求扩展中
    req.extensions_mut().insert(AuthClaims(claims));

    Ok(next.run(req).await)
}

/// User authentication middleware.
///
/// Verifies JWT token for user role.
//
// // 用户身份验证中间件。
// //
// // 验证用户角色的 JWT 令牌。
pub async fn require_user(
    State(jwt_manager): State<Arc<JwtManager>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 Authorization 头
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    // 2. 提取 Bearer 令牌
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

    // 3. 验证令牌
    let claims = jwt_manager.verify_token(token).await?;

    // 4. 检查角色
    if claims.role != "user" {
        return Err(AppError::Forbidden("User access required".to_string()));
    }

    // 5. 将声明添加到请求扩展中
    req.extensions_mut().insert(AuthClaims(claims));

    Ok(next.run(req).await)
}
