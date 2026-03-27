// Main router assembly
//
// // 主路由组装器

use axum::{Router, routing::post};
use std::sync::Arc;
use crate::config::Config;
use crate::utils::{jwt::JwtManager, rate_limiter::RateLimiter};
use super::{public, static_files, admin};

/// Creates the main application router by combining all sub-routers.
//
// // 通过组合所有子路由器创建主应用程序路由器。
pub fn create_router(config: Config, jwt_manager: Arc<JwtManager>) -> Router {
    // 1. 创建管理员状态（5分钟内最多10次请求）
    let admin_state = admin::AdminState {
        admin_config: config.admin.clone(),
        jwt_manager: jwt_manager.clone(),
        rate_limiter: RateLimiter::new(300, 10),
    };

    Router::new()
        // 2. 管理员登录路由
        .route("/api/admin/login", post(admin::admin_login))
        .with_state(admin_state)
        // 3. 公共路由
        .merge(public::routes(config.site_info))
        .merge(static_files::routes(config.storage.static_path, config.storage.files_path))
        // 未来在此添加其他路由模块
        // .merge(user::routes())
        // .merge(article::routes())
        // .merge(file::routes())
}
