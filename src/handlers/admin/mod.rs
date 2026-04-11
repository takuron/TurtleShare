// Admin handlers module - Admin-only API endpoints
//
// // 管理员处理器模块 - 仅管理员可用的 API 端点

pub mod auth;
pub mod users;
pub mod subscriptions;
pub mod articles;
pub mod files;
pub mod announcement;

pub use auth::AdminState;
