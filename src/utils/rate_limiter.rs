use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Sliding window rate limiter.
///
/// Tracks requests per IP address using a sliding time window.
//
// // 滑动窗口限流器。
// //
// // 使用滑动时间窗口跟踪每个 IP 地址的请求。
#[derive(Clone)]
pub struct RateLimiter {
    // IP -> (timestamps of requests)
    records: Arc<Mutex<HashMap<String, Vec<u64>>>>,
    window_secs: u64,
    max_requests: usize,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Arguments
    /// * `window_secs` - Time window in seconds
    /// * `max_requests` - Maximum requests allowed in the window
    //
    // // 创建新的限流器。
    // //
    // // # 参数
    // // * `window_secs` - 时间窗口（秒）
    // // * `max_requests` - 窗口内允许的最大请求数
    pub fn new(window_secs: u64, max_requests: usize) -> Self {
        Self {
            records: Arc::new(Mutex::new(HashMap::new())),
            window_secs,
            max_requests,
        }
    }

    /// Checks if a request from the given IP should be allowed.
    ///
    /// # Arguments
    /// * `ip` - The IP address to check
    ///
    /// # Returns
    /// Returns true if the request is allowed, false if rate limited.
    //
    // // 检查来自给定 IP 的请求是否应被允许。
    // //
    // // # 参数
    // // * `ip` - 要检查的 IP 地址
    // //
    // // # 返回
    // // 如果请求被允许则返回 true，如果被限流则返回 false。
    pub async fn check(&self, ip: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut records = self.records.lock().await;

        // 1. 获取或创建该 IP 的记录
        let timestamps = records.entry(ip.to_string()).or_insert_with(Vec::new);

        // 2. 移除窗口外的旧记录
        let cutoff = now.saturating_sub(self.window_secs);
        timestamps.retain(|&t| t > cutoff);

        // 3. 检查是否超过限制
        if timestamps.len() >= self.max_requests {
            if timestamps.len() == self.max_requests {
                tracing::warn!(
                    "Rate limit triggered for IP: {} (max {} requests per {}s)",
                    ip,
                    self.max_requests,
                    self.window_secs
                );
            }
            // 记录被拒绝的请求，直到 1.5 倍的 max_requests，从而防止重复打印日志且拥有更多记录控制
            if timestamps.len() < self.max_requests + self.max_requests / 2 {
                timestamps.push(now);
            }
            return false;
        }

        // 4. 记录本次请求
        timestamps.push(now);
        true
    }
}
