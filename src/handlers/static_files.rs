// Static file serving
//
// // 静态文件服务

use axum::Router;
use tower_http::services::ServeDir;

/// Creates routes for serving static files.
///
/// # Arguments
/// * `static_path` - Path to the static files directory (frontend)
/// * `files_path` - Path to the uploaded files directory
//
// // 创建静态文件服务路由。
// //
// // # 参数
// // * `static_path` - 静态文件目录路径（前端）
// // * `files_path` - 上传文件目录路径
pub fn routes(static_path: String, files_path: String) -> Router {
    Router::new()
        .nest_service("/files", ServeDir::new(files_path))
        .fallback_service(
            ServeDir::new(static_path)
                .append_index_html_on_directories(true)
        )
}
