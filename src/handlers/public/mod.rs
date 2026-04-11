// Public handlers module - Public API endpoints and static file serving
//
// // 公开处理器模块 - 公开 API 端点和静态文件服务

pub mod api;
pub mod articles;
pub mod announcement;
pub mod tier_descriptions;

use crate::config::SiteInfoConfig;
use crate::utils::hashid::HashIdManager;
use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Shared state for all public handlers.
///
/// Contains the database pool and HashID manager needed by public endpoints.
//
// // 所有公开处理器的共享状态。
// //
// // 包含公开端点所需的数据库连接池和 HashID 管理器。
#[derive(Clone)]
pub struct PublicState {
    pub pool: SqlitePool,
    pub hashid_manager: Arc<HashIdManager>,
}

/// Creates all public API routes using a shared state.
///
/// # Arguments
/// * `siteinfo` - Site information configuration
/// * `state` - Shared public state containing pool and hashid_manager
//
// // 使用共享状态创建所有公开 API 路由。
// //
// // # 参数
// // * `siteinfo` - 站点信息配置
// // * `state` - 包含连接池和 HashID 管理器的共享公开状态
pub fn routes(siteinfo: SiteInfoConfig, state: PublicState) -> Router {
    // 1. 将 toml::Table 转换为 serde_json::Value，以便 JSON 序列化。
    let siteinfo_json: serde_json::Value =
        serde_json::to_value(&siteinfo).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    Router::new()
        .route("/api", get(api::health_check_text))
        .route("/api/health", get(api::health_check))
        .route(
            "/api/public/site-info",
            get({
                move || async move {
                    Json(crate::handlers::common::ApiResponse {
                        success: true,
                        data: siteinfo_json,
                    })
                }
            }),
        )
        .route(
            "/api/public/announcement",
            get(announcement::get_announcement),
        )
        .route(
            "/api/public/tier-descriptions",
            get(tier_descriptions::get_tier_descriptions),
        )
        .route("/api/public/articles", get(articles::list_articles))
        .route(
            "/api/public/articles/page",
            get(articles::get_articles_page_count),
        )
        .route(
            "/api/public/articles/page/{page}",
            get(articles::list_articles_paginated),
        )
        .route("/api/public/articles/{hash_id}", get(articles::get_article))
        .with_state(state)
}

use axum::Json;
use axum::routing::get;
