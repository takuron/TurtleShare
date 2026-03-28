// 管理员身份验证集成测试
//
// 测试 /api/admin/login 端点以及 JWT 认证中间件的安全行为。
// 包含正常登录、错误凭据、恶意请求等场景。

use super::common;
use serde_json::{json, Value};

// ============================================================
// 正常登录流程
// ============================================================

/// 使用正确凭据登录应返回 200 和有效的 JWT 令牌。
#[tokio::test]
async fn admin_login_success() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // 令牌应该是非空字符串
    let token = body["data"]["token"].as_str().unwrap();
    assert!(!token.is_empty());
    // JWT 格式：header.payload.signature
    assert_eq!(token.matches('.').count(), 2);
}

/// 登录后获取的令牌应能访问受保护的管理员端点。
#[tokio::test]
async fn admin_token_grants_access_to_protected_routes() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server.get_with_token("/api/admin/users", &token).await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
}

// ============================================================
// 错误凭据
// ============================================================

/// 错误密码应返回 401。
#[tokio::test]
async fn admin_login_wrong_password() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "wrong_password"}),
        )
        .await;

    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

/// 错误用户名应返回 401。
#[tokio::test]
async fn admin_login_wrong_username() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "not_admin", "password": "admin123"}),
        )
        .await;

    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
}

/// 用户名和密码都错误应返回 401。
#[tokio::test]
async fn admin_login_both_wrong() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "hacker", "password": "letmein"}),
        )
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 恶意/畸形请求
// ============================================================

/// 空请求体应返回 400。
#[tokio::test]
async fn admin_login_empty_body() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .post(server.url("/api/admin/login"))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .unwrap();

    // 缺少必填字段，应返回 400
    assert_eq!(resp.status(), 400);
}

/// 非 JSON 请求体应返回 400。
#[tokio::test]
async fn admin_login_invalid_json() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .post(server.url("/api/admin/login"))
        .header("Content-Type", "application/json")
        .body("this is not json")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

/// 无 Content-Type 的请求应返回 400。
#[tokio::test]
async fn admin_login_no_content_type() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .post(server.url("/api/admin/login"))
        .body(r#"{"username":"admin","password":"admin123"}"#)
        .send()
        .await
        .unwrap();

    // 没有 Content-Type: application/json，axum 应拒绝
    assert_eq!(resp.status(), 400);
}

/// SQL 注入尝试应被安全处理（返回 401，不崩溃）。
#[tokio::test]
async fn admin_login_sql_injection_attempt() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({
                "username": "admin' OR '1'='1",
                "password": "' OR '1'='1' --"
            }),
        )
        .await;

    // 不应崩溃，应返回 401
    assert_eq!(resp.status(), 401);
}

/// 超长用户名/密码不应导致服务器崩溃。
#[tokio::test]
async fn admin_login_extremely_long_credentials() {
    let server = common::TestServer::spawn().await;

    let long_string = "A".repeat(100_000);
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": long_string, "password": long_string}),
        )
        .await;

    // 应返回 401 而不是 500 或超时
    let status = resp.status().as_u16();
    assert!(status == 401 || status == 400 || status == 413);
}

/// 包含额外未知字段的请求应正常处理（忽略多余字段）。
#[tokio::test]
async fn admin_login_extra_fields_ignored() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({
                "username": "admin",
                "password": "admin123",
                "evil_field": "drop table users;",
                "role": "admin"
            }),
        )
        .await;

    // 多余字段应被忽略，正常登录
    assert_eq!(resp.status(), 200);
}

// ============================================================
// 未认证访问受保护端点
// ============================================================

/// 无令牌访问受保护端点应返回 401。
#[tokio::test]
async fn protected_route_without_token() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/users").await;
    assert_eq!(resp.status(), 401);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

/// 伪造的令牌应返回 401。
#[tokio::test]
async fn protected_route_with_fake_token() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .get_with_token("/api/admin/users", "fake.jwt.token")
        .await;

    assert_eq!(resp.status(), 401);
}

/// 格式错误的 Authorization 头应返回 401。
#[tokio::test]
async fn protected_route_with_malformed_auth_header() {
    let server = common::TestServer::spawn().await;

    // 使用 "Basic" 而不是 "Bearer"
    let resp = server
        .client
        .get(server.url("/api/admin/users"))
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// 空的 Bearer 令牌应返回 401。
#[tokio::test]
async fn protected_route_with_empty_bearer() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .get(server.url("/api/admin/users"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// 手工构造的过期令牌应被拒绝。
#[tokio::test]
async fn expired_token_is_rejected() {
    let server = common::TestServer::spawn().await;

    // 使用一个看起来像 JWT 但已过期的伪造令牌
    // 由于我们不知道服务器的签名密钥，任何手工构造的令牌都会验证失败
    // 这里测试的是：服务器不会因为过期令牌而崩溃，且正确返回 401
    let fake_expired = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiJhZG1pbiIsIm5hbWUiOiJhZG1pbiIsInJvbGUiOiJhZG1pbiIsImV4cCI6MTAwMDAwMDAwMCwiaWF0IjoxMDAwMDAwMDAwfQ.\
        invalid_signature";

    let resp = server
        .get_with_token("/api/admin/users", fake_expired)
        .await;

    assert_eq!(resp.status(), 401);
}
