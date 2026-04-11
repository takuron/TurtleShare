// Public API endpoints (no authentication required)
//
// // 公开 API 端点（无需身份验证）

use crate::handlers::common::ApiResponse;
use axum::Json;

/// Simple API running indicator endpoint.
//
// // 简单的 API 运行指示端点。
pub async fn health_check_text() -> &'static str {
    "TurtleShare API is running!"
}

/// Simple health check endpoint.
//
// // 简单的健康检查端点。
pub async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "status": "ok" }),
    })
}
