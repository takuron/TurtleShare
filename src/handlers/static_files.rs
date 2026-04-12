// Static file serving
//
// // 静态文件服务

use axum::http::{StatusCode, Uri, header};
use axum::response::IntoResponse;
use axum::Router;
use tower_http::services::ServeDir;

/// Creates routes for serving static files.
///
/// Handles SvelteKit's static adapter output by trying the `.html` extension
/// when a direct file is not found (e.g., `/admin` → `admin.html`).
///
/// # Arguments
/// * `static_path` - Path to the static files directory (frontend)
/// * `files_path` - Path to the uploaded files directory
//
// // 创建静态文件服务路由。
// //
// // SvelteKit 静态适配器将路由输出为 {route}.html 文件。
// // 当直接路径未找到时，自动尝试 .html 扩展名（如 /admin → admin.html）。
// //
// // # 参数
// // * `static_path` - 静态文件目录路径（前端）
// // * `files_path` - 上传文件目录路径
pub fn routes(static_path: String, files_path: String) -> Router {
    let static_dir = std::path::PathBuf::from(&static_path);

    // SvelteKit 静态适配器将页面输出为 {route}.html，
    // 但 ServeDir 不会自动尝试 .html 扩展名，导致刷新子页面时 404。
    // 此回退服务在原始路径未命中时追加 .html 再次查找。
    let html_fallback = Router::new().fallback(move |uri: Uri| {
        let static_dir = static_dir.clone();
        async move {
            let path = uri.path().trim_start_matches('/');

            // 防止路径穿越攻击
            if path.contains("..") {
                return StatusCode::NOT_FOUND.into_response();
            }

            // 尝试追加 .html 扩展名（如 /admin → admin.html）
            let html_file = static_dir.join(format!("{}.html", path));
            match tokio::fs::read(&html_file).await {
                Ok(contents) => (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    contents,
                )
                    .into_response(),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    Router::new()
        .nest_service("/files", ServeDir::new(files_path))
        .fallback_service(
            ServeDir::new(static_path)
                .append_index_html_on_directories(true)
                .not_found_service(html_fallback),
        )
}
