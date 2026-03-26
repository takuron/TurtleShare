// Public endpoints (no authentication required)
//
// // 公开端点（无需身份验证）

use axum::{routing::get, Json, Router};
use crate::config::SiteInfoConfig;
use super::common::ApiResponse;

/// Creates the public routes.
//
// // 创建公开路由。
pub fn routes(site_info: SiteInfoConfig) -> Router {
    Router::new()
        .route("/api", get(|| async { "TurtleShare API is running!" }))
        .route("/api/health", get(health_check))
        .route("/api/public/site-info", get({
            move || async move {
                Json(ApiResponse {
                    success: true,
                    data: site_info
                })
            }
        }))
}

/// Simple health check endpoint.
//
// // 简单的健康检查端点。
async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "status": "ok" }),
    })
}
