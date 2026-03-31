// Main router assembly
//
// // 主路由组装器

use super::admin;
use super::public;
use crate::config::Config;
use crate::utils::{hashid::HashIdManager, jwt::JwtManager, rate_limiter::RateLimiter};
use axum::{
    Router,
    routing::{get, post, put},
};
use std::sync::Arc;

/// Creates the main application router by combining all sub-routers.
//
// // 通过组合所有子路由器创建主应用程序路由器。
pub fn create_router(
    config: Config,
    jwt_manager: Arc<JwtManager>,
    hashid_manager: Arc<HashIdManager>,
    pool: sqlx::SqlitePool,
) -> Router {
    // 1. 创建管理员状态（5分钟内最多10次请求）
    let admin_state = admin::AdminState {
        admin_config: config.admin.clone(),
        jwt_manager: jwt_manager.clone(),
        hashid_manager: hashid_manager.clone(),
        rate_limiter: RateLimiter::new(300, 10),
        pool,
        files_path: config.storage.files_path.clone(),
        max_upload_size_bytes: config.storage.max_upload_size_mb * 1024 * 1024,
        base_url: config.server.base_url.clone(),
    };

    let admin_protected = Router::new()
        .route("/users", get(admin::users::list_users).post(admin::users::create_user))
        .route(
            "/users/{hash_id}",
            get(admin::users::get_user)
                .put(admin::users::update_user)
                .delete(admin::users::delete_user),
        )
        .route("/users/{hash_id}/tier", get(admin::users::get_user_tier))
        .route(
            "/users/{hash_id}/subscriptions",
            get(admin::subscriptions::list_user_subscriptions)
                .post(admin::subscriptions::create_subscription),
        )
        .route(
            "/subscriptions/{hash_id}",
            put(admin::subscriptions::update_subscription)
                .delete(admin::subscriptions::delete_subscription),
        )
        .route(
            "/articles",
            get(admin::articles::list_articles).post(admin::articles::create_article),
        )
        .route(
            "/articles/{hash_id}",
            get(admin::articles::get_article)
                .put(admin::articles::update_article)
                .delete(admin::articles::delete_article),
        )
        .route(
            "/files",
            get(admin::files::list_files).post(admin::files::upload_file),
        )
        .route(
            "/files/{hash_id}",
            get(admin::files::get_file).delete(admin::files::delete_file),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_manager.clone(),
            crate::middleware::auth::require_admin,
        ))
        .with_state(admin_state.clone());

    Router::new()
        // 2. 管理员登录路由
        .route("/api/admin/login", post(admin::auth::admin_login))
        .with_state(admin_state)
        // 3. 保护的管理员路由
        .nest("/api/admin", admin_protected)
        // 4. 公共路由
        .merge(public::api::routes(config.site_info))
        .merge(super::static_files::routes(
            config.storage.static_path,
            config.storage.files_path,
        ))
    // 未来在此添加其他路由模块
    // .merge(user::routes())
}
