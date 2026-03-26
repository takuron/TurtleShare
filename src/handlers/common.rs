// Common types and utilities for handlers
//
// // 处理器的通用类型和工具

use serde::Serialize;

/// Standard API response wrapper.
//
// // 标准 API 响应包装。
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}
