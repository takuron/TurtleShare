// Main router assembly.

use super::admin;
use super::public;
use super::user;
use crate::config::Config;
use crate::error::Result;
use crate::middleware::cors::{enforce_cors, CorsPolicy};
use crate::middleware::rate_limiter::global_rate_limit;
use crate::utils::{hashid::HashIdManager, jwt::JwtManager, rate_limiter::RateLimiter};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post, put},
    Extension, Router,
};
use std::sync::Arc;

/// Creates the main application router by combining all sub-routers.
///
/// # Arguments
/// * `config` - The loaded application configuration.
/// * `jwt_manager` - Shared JWT manager used by authentication middleware.
/// * `hashid_manager` - Shared HashID manager for external IDs.
/// * `pool` - Shared SQLite connection pool.
///
/// # Returns
/// Returns the fully configured Axum `Router`.
///
/// # Errors
/// Returns `AppError::Config` if the CORS configuration derived from
/// `config.server` is invalid.
pub fn create_router(
    config: Config,
    jwt_manager: Arc<JwtManager>,
    hashid_manager: Arc<HashIdManager>,
    pool: sqlx::SqlitePool,
) -> Result<Router> {
    let global_limiter = Arc::new(RateLimiter::new(60, 100));
    let cors_policy = Arc::new(CorsPolicy::from_server_config(&config.server)?);

    let admin_state = admin::AdminState {
        admin_config: config.admin.clone(),
        jwt_manager: jwt_manager.clone(),
        hashid_manager: hashid_manager.clone(),
        rate_limiter: RateLimiter::new(300, 10),
        pool: pool.clone(),
        files_path: config.storage.files_path.clone(),
        max_upload_size_bytes: config.storage.max_upload_size_mb * 1024 * 1024,
        base_url: config.server.base_url.clone(),
    };

    let user_state = user::UserState {
        jwt_manager: jwt_manager.clone(),
        hashid_manager: hashid_manager.clone(),
        rate_limiter: RateLimiter::new(300, 10),
        pool: pool.clone(),
    };

    let max_upload_bytes = admin_state.max_upload_size_bytes as usize;

    let admin_protected = Router::new()
        .route(
            "/users",
            get(admin::users::list_users).post(admin::users::create_user),
        )
        .route("/users/page", get(admin::users::get_users_page_count))
        .route(
            "/users/page/{page}",
            get(admin::users::list_users_paginated),
        )
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
            "/articles/page",
            get(admin::articles::get_articles_page_count),
        )
        .route(
            "/articles/page/{page}",
            get(admin::articles::list_articles_paginated),
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
        .route("/files/page", get(admin::files::get_files_page_count))
        .route(
            "/files/page/{page}",
            get(admin::files::list_files_paginated),
        )
        .route(
            "/files/{hash_id}",
            get(admin::files::get_file).delete(admin::files::delete_file),
        )
        .route(
            "/announcement",
            put(admin::announcement::publish_announcement),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_manager.clone(),
            crate::middleware::auth::require_admin,
        ))
        .layer(DefaultBodyLimit::max(max_upload_bytes))
        .with_state(admin_state.clone());

    let user_protected = Router::new()
        .route("/password", put(user::operations::change_password))
        .route(
            "/subscriptions",
            get(user::operations::get_own_subscriptions),
        )
        .route("/tier", get(user::operations::get_own_tier))
        .route("/articles", get(user::articles::list_articles))
        .route(
            "/articles/page",
            get(user::articles::get_articles_page_count),
        )
        .route(
            "/articles/page/{page}",
            get(user::articles::list_articles_paginated),
        )
        .route("/articles/{hash_id}", get(user::articles::get_article))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_manager.clone(),
            crate::middleware::auth::require_user,
        ))
        .with_state(user_state.clone());

    let api_router = Router::new()
        .route("/api/admin/login", post(admin::auth::admin_login))
        .with_state(admin_state.clone())
        .nest("/api/admin", admin_protected)
        .route("/api/users/login", post(user::auth::user_login))
        .with_state(user_state.clone())
        .nest("/api/users", user_protected)
        .merge(public::api::routes(
            config.siteinfo,
            pool.clone(),
            hashid_manager,
        ))
        .merge(public::announcement::routes(pool))
        .layer(axum::middleware::from_fn(global_rate_limit))
        .layer(Extension(global_limiter));

    Ok(Router::new()
        .merge(api_router)
        .merge(super::static_files::routes(
            config.storage.static_path,
            config.storage.files_path,
        ))
        .layer(axum::middleware::from_fn_with_state(
            cors_policy,
            enforce_cors,
        )))
}
