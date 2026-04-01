// Public API endpoints (no authentication required)
//
// // 公开 API 端点（无需身份验证）

use crate::config::SiteInfoConfig;
use crate::handlers::common::ApiResponse;
use crate::handlers::public::articles::{self, PublicArticleState};
use crate::utils::hashid::HashIdManager;
use axum::{Json, Router, routing::get};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Creates the public API routes.
///
/// # Arguments
/// * `site_info` - Site information configuration
/// * `pool` - Database connection pool
/// * `hashid_manager` - HashID manager for encoding/decoding IDs
//
// // 创建公开 API 路由。
// //
// // # 参数
// // * `site_info` - 站点信息配置
// // * `pool` - 数据库连接池
// // * `hashid_manager` - 用于编码/解码 ID 的 HashID 管理器
pub fn routes(
    site_info: SiteInfoConfig,
    pool: SqlitePool,
    hashid_manager: Arc<HashIdManager>,
) -> Router {
    // 1. 创建公开文章状态
    let public_article_state = PublicArticleState {
        pool,
        hashid_manager,
    };

    Router::new()
        .route("/api", get(|| async { "TurtleShare API is running!" }))
        .route("/api/health", get(health_check))
        .route(
            "/api/public/site-info",
            get({
                move || async move {
                    Json(ApiResponse {
                        success: true,
                        data: site_info,
                    })
                }
            }),
        )
        // 2. 公开文章路由
        .route("/api/public/articles", get(articles::list_articles))
        .route("/api/public/articles/{hash_id}", get(articles::get_article))
        .with_state(public_article_state)
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
