// Handlers module - HTTP request handlers
//
// // 处理器模块 - HTTP 请求处理器

pub mod admin;
pub mod common;
pub mod public;
pub mod routes;
pub mod static_files;
pub mod user;

pub use routes::create_router;
