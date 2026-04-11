// 系统级黑箱集成测试入口
//
// 所有通过 HTTP 交互的端到端测试模块在此注册。
// 每个模块对应一组 API 端点的测试。

mod admin_articles;
mod admin_auth;
mod admin_files;
mod admin_subscriptions;
mod admin_users;
mod announcement;
mod common;
mod cors;
mod health_check;
mod pagination;
mod public_articles;
mod publish_at;
mod rate_limiter;
mod user_articles;
mod user_auth;
mod user_operations;
