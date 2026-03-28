// 健康检查 API 集成测试
//
// 测试 /api/health 和 /api 端点是否按照 docs/api.md 的规范正确响应。

use super::common;
use serde_json::Value;

/// Tests that GET /api/health returns 200 with {"success": true, "data": {"status": "ok"}}.
//
// // 测试 GET /api/health 返回 200 和 {"success": true, "data": {"status": "ok"}}。
#[tokio::test]
async fn health_check_returns_ok() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/health").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体结构
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "ok");
}

/// Tests that GET /api returns the plain text status message.
//
// // 测试 GET /api 返回纯文本状态消息。
#[tokio::test]
async fn api_root_returns_running_message() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应内容
    let text = resp.text().await.unwrap();
    assert_eq!(text, "TurtleShare API is running!");
}

/// Tests that GET /api/public/site-info returns configured site information.
//
// // 测试 GET /api/public/site-info 返回配置的站点信息。
#[tokio::test]
async fn site_info_returns_configured_values() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/site-info").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体包含配置中的站点信息
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "TurtleShare-Test");
    assert_eq!(body["data"]["author"], "TestAdmin");
    assert!(body["data"]["base_url"].as_str().unwrap().contains(&server.port.to_string()));
}

/// Tests that requesting a non-existent API path returns 404.
//
// // 测试请求不存在的 API 路径返回 404。
#[tokio::test]
async fn nonexistent_api_path_returns_404() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/nonexistent").await;

    // 注意：由于 SPA fallback，非 /api 路径可能返回 200
    // 但 /api 前缀下不存在的路径应该返回 404
    assert_eq!(resp.status(), 404);
}
