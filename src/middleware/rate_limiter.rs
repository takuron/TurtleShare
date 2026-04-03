use crate::error::AppError;
use crate::utils::rate_limiter::RateLimiter;
use axum::{
    extract::{ConnectInfo, Extension, Request},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// Global rate limiting middleware.
///
/// Applies a sliding window rate limit of 500 requests per 5 minutes per IP address
/// to all API endpoints.
///
/// Uses `Extension` instead of `State` to avoid conflicts with the router's primary
/// state type, allowing this middleware to be layered on top of routes that use
/// different state types (e.g., `AdminState`, `UserState`).
///
/// # Arguments
/// * `limiter` - Shared rate limiter state (passed via Extension)
/// * `addr` - Client socket address
/// * `req` - Incoming HTTP request
/// * `next` - Next middleware/handler in the chain
///
/// # Returns
/// Returns the response from the next handler, or a 429 Too Many Requests error.
//
// // 全局限流中间件。
// //
// // 对所有 API 端点应用每 IP 每 5 分钟最多 500 次请求的滑动窗口限制。
// //
// // 使用 `Extension` 而不是 `State` 以避免与路由器的主要状态类型冲突，
// // 允许此中间件应用于使用不同状态类型（如 `AdminState`、`UserState`）的路由之上。
// //
// // # 参数
// // * `limiter` - 共享的限流器状态（通过 Extension 传递）
// // * `addr` - 客户端套接字地址
// // * `req` - 传入的 HTTP 请求
// // * `next` - 链中的下一个中间件/处理器
// //
// // # 返回
// // 返回下一个处理器的响应，或返回 429 请求过多错误。
pub async fn global_rate_limit(
    Extension(limiter): Extension<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取客户端 IP 地址
    let ip = addr.ip().to_string();

    // 2. 检查限流状态
    if !limiter.check(&ip).await {
        // 3. 如果超过限制，返回 429 错误
        return Err(AppError::TooManyRequests(format!(
            "Rate limit exceeded for IP {}. Maximum 500 requests per 5 minutes.",
            ip
        )));
    }

    // 4. 继续处理请求
    Ok(next.run(req).await)
}
