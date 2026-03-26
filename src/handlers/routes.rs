// Main router assembly
//
// // 主路由组装器

use axum::Router;
use crate::config::{SiteInfoConfig, StorageConfig};
use super::{public, static_files};

/// Creates the main application router by combining all sub-routers.
//
// // 通过组合所有子路由器创建主应用程序路由器。
pub fn create_router(site_info: SiteInfoConfig, storage: StorageConfig) -> Router {
    Router::new()
        .merge(public::routes(site_info))
        .merge(static_files::routes(storage.static_path, storage.files_path))
        // 未来在此添加其他路由模块
        // .merge(admin::routes())
        // .merge(user::routes())
        // .merge(article::routes())
        // .merge(file::routes())
}
