// Handlers module - HTTP request handlers
//
// // 处理器模块 - HTTP 请求处理器

pub mod common;
pub mod admin;
pub mod user;
pub mod public;
pub mod static_files;
pub mod routes;

pub use routes::create_router;
pub use common::ApiResponse;
