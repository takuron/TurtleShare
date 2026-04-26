// Common types and utilities for handlers
//
// // 处理器的通用类型和工具

use serde::{Deserialize, Serialize};

/// Standard API response wrapper.
//
// // 标准 API 响应包装。
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

/// Common query parameters for pagination.
//
// // 用于分页的常见查询参数。
#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page_size: Option<u32>,
}

/// Common query parameters for search.
//
// // 用于搜索的常见查询参数。
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub page_size: Option<u32>,
}

/// Response containing pagination count metadata.
//
// // 包含分页计数元数据的响应。
#[derive(Serialize)]
pub struct PageCountResponse {
    pub total_pages: u32,
    pub total_items: u32,
}
