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
/// Applies a sliding window rate limit of 100 requests per 1 minute per IP address
/// to all API endpoints.
///
/// The client IP is resolved in this order:
/// 1. Value of the `X-Real-IP` header (set by the outermost reverse proxy, usually the real public IP).
/// 2. First non-empty token from the `X-Forwarded-For` header (first hop may be an internal IP in multi-proxy setups).
/// 3. TCP connection peer address as a final fallback.
///
/// Uses `Extension` instead of `State` to avoid conflicts with the router's primary
/// state type, allowing this middleware to be layered on top of routes that use
/// different state types (e.g., `AdminState`, `UserState`).
///
/// # Arguments
/// * `limiter` - Shared rate limiter state (passed via Extension)
/// * `addr` - Client socket address (TCP fallback)
/// * `req` - Incoming HTTP request
/// * `next` - Next middleware/handler in the chain
///
/// # Returns
/// Returns the response from the next handler, or a 429 Too Many Requests error.
//
// // 全局限流中间件。
// //
// // 对所有 API 端点应用每 IP 每 1 分钟最多 100 次请求的滑动窗口限制。
// //
// // 客户端 IP 按以下优先级解析：
// // 1. X-Real-IP 头的值（由最外层反向代理设置，通常是真实公网 IP）。
// // 2. X-Forwarded-For 头中第一个非空令牌（多级代理时首跳可能是内网 IP）。
// // 3. 最终回退到 TCP 连接的对端地址。
// //
// // 使用 `Extension` 而不是 `State` 以避免与路由器的主要状态类型冲突，
// // 允许此中间件应用于使用不同状态类型（如 `AdminState`、`UserState`）的路由之上。
// //
// // # 参数
// // * `limiter` - 共享的限流器状态（通过 Extension 传递）
// // * `addr` - 客户端套接字地址（TCP 回退）
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
    // 1. 优先读取 X-Real-IP 头（由最外层反代设置，通常是真实客户端公网 IP）
    let ip = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        // 2. 其次读取 X-Forwarded-For 头，取第一个 IP（多级代理时首跳可能是内网 IP）
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
        })
        // 3. 最终回退到 TCP 连接的对端 IP（直连场景）
        .unwrap_or_else(|| addr.ip().to_string());

    // 4. 检查限流状态
    if !limiter.check(&ip).await {
        // 5. 超过限制，返回 429 错误
        return Err(AppError::TooManyRequests("Rate limit exceeded".to_string()));
    }

    // 6. 继续处理请求
    Ok(next.run(req).await)
}
