// 用户认证 API 集成测试
//
// 测试 /api/users/login 端点的完整行为。
// 包含正常登录、错误凭据、JSON 格式错误和安全测试。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 通过管理员 API 创建一个用户，返回其 hash_id。
async fn create_test_user(
    server: &common::TestServer,
    admin_token: &str,
    username: &str,
    password: &str,
) -> String {
    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": username,
                "password": password,
                "email": format!("{}@test.com", username),
                "note": format!("Test user {}", username)
            }),
            admin_token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create user {}", username);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

/// 以用户身份登录，返回 JWT 令牌。
async fn user_login(
    server: &common::TestServer,
    username: &str,
    password: &str,
) -> reqwest::Response {
    server
        .post_json(
            "/api/users/login",
            &json!({
                "username": username,
                "password": password
            }),
        )
        .await
}

// ============================================================
// 用户登录 - 成功场景
// ============================================================

/// 使用正确凭据登录应返回 200 和 JWT 令牌。
#[tokio::test]
async fn user_login_success() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 1. 通过管理员 API 创建用户
    create_test_user(&server, &admin_token, "login_user", "user_pass_123").await;

    // 2. 使用用户凭据登录
    let resp = user_login(&server, "login_user", "user_pass_123").await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // 验证令牌存在且非空
    let token = body["data"]["token"].as_str().unwrap();
    assert!(!token.is_empty());

    // 验证响应中不包含密码相关信息
    assert!(body["data"]["password"].is_null());
    assert!(body["data"]["password_hash"].is_null());
}

/// 登录返回的令牌应可以访问用户保护端点。
#[tokio::test]
async fn user_login_token_is_valid() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "token_user", "token_pass").await;

    // 登录获取令牌
    let resp = user_login(&server, "token_user", "token_pass").await;
    let body: Value = resp.json().await.unwrap();
    let user_token = body["data"]["token"].as_str().unwrap();

    // 使用令牌访问用户保护端点
    let sub_resp = server
        .get_with_token("/api/users/subscriptions", user_token)
        .await;
    assert_eq!(sub_resp.status(), 200);
}

/// 用户令牌不应能访问管理员端点。
#[tokio::test]
async fn user_token_cannot_access_admin_routes() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "no_admin_user", "pass123").await;

    // 以用户身份登录
    let resp = user_login(&server, "no_admin_user", "pass123").await;
    let body: Value = resp.json().await.unwrap();
    let user_token = body["data"]["token"].as_str().unwrap();

    // 尝试访问管理员端点
    let admin_resp = server
        .get_with_token("/api/admin/users", user_token)
        .await;
    assert_eq!(admin_resp.status(), 403);
}

/// 管理员令牌不应能访问用户保护端点。
#[tokio::test]
async fn admin_token_cannot_access_user_routes() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 管理员令牌的 role 为 "admin"，用户路由要求 role 为 "user"
    let resp = server
        .get_with_token("/api/users/subscriptions", &admin_token)
        .await;
    assert_eq!(resp.status(), 403);
}

// ============================================================
// 用户登录 - 错误场景
// ============================================================

/// 错误密码应返回 401。
#[tokio::test]
async fn user_login_wrong_password() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "wrong_pass_user", "correct_pass").await;

    let resp = user_login(&server, "wrong_pass_user", "wrong_password").await;

    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

/// 不存在的用户应返回 401。
#[tokio::test]
async fn user_login_nonexistent_user() {
    let server = common::TestServer::spawn().await;

    let resp = user_login(&server, "ghost_user", "any_password").await;

    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

/// 无效的 JSON 格式应返回 400。
#[tokio::test]
async fn user_login_invalid_json() {
    let server = common::TestServer::spawn().await;

    // 发送非 JSON 内容
    let resp = server
        .client
        .post(server.url("/api/users/login"))
        .header("Content-Type", "application/json")
        .body("not valid json")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 空请求体应返回 400。
#[tokio::test]
async fn user_login_empty_body() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .post(server.url("/api/users/login"))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .unwrap();

    // 缺少必填字段应该返回错误
    let status = resp.status().as_u16();
    assert!(status == 400 || status == 422, "Expected 400 or 422, got {}", status);
}

// ============================================================
// 认证中间件测试
// ============================================================

/// 无 Authorization 头访问保护端点应返回 401。
#[tokio::test]
async fn user_protected_without_auth() {
    let server = common::TestServer::spawn().await;

    // 不带令牌访问各用户保护端点
    assert_eq!(server.get("/api/users/subscriptions").await.status(), 401);

    let resp = server
        .put_json(
            "/api/users/password",
            &json!({"current_password": "x", "new_password": "y"}),
        )
        .await;
    assert_eq!(resp.status(), 401);
}

/// 格式错误的 Bearer 令牌应返回 401。
#[tokio::test]
async fn user_protected_malformed_token() {
    let server = common::TestServer::spawn().await;

    // 使用非法令牌
    let resp = server
        .get_with_token("/api/users/subscriptions", "not-a-valid-jwt-token")
        .await;
    assert_eq!(resp.status(), 401);
}

/// 空 Bearer 令牌应返回 401。
#[tokio::test]
async fn user_protected_empty_bearer() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .get(server.url("/api/users/subscriptions"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// 无 Bearer 前缀的 Authorization 头应返回 401。
#[tokio::test]
async fn user_protected_no_bearer_prefix() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .get(server.url("/api/users/subscriptions"))
        .header("Authorization", "Token abc123")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 安全性测试
// ============================================================

/// SQL 注入尝试：用户名中包含 SQL 语句不应泄露信息。
#[tokio::test]
async fn user_login_sql_injection_in_username() {
    let server = common::TestServer::spawn().await;

    let resp = user_login(&server, "' OR '1'='1' --", "any_password").await;

    // 应返回 401，不应触发 500 或泄露信息
    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

/// SQL 注入尝试：密码中包含 SQL 语句不应影响数据库。
#[tokio::test]
async fn user_login_sql_injection_in_password() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "sqli_pass_user", "normal_pass").await;

    let resp = user_login(&server, "sqli_pass_user", "' OR '1'='1' --").await;

    // SQL 注入不应绕过密码验证
    assert_eq!(resp.status(), 401);
}

/// 已删除用户不应能登录。
#[tokio::test]
async fn deleted_user_cannot_login() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id =
        create_test_user(&server, &admin_token, "deleted_user", "del_pass_123").await;

    // 确认可以登录
    let resp = user_login(&server, "deleted_user", "del_pass_123").await;
    assert_eq!(resp.status(), 200);

    // 管理员删除用户
    let del_resp = server
        .delete_with_token(&format!("/api/admin/users/{}", hash_id), &admin_token)
        .await;
    assert_eq!(del_resp.status(), 200);

    // 尝试再次登录
    let resp2 = user_login(&server, "deleted_user", "del_pass_123").await;
    assert_eq!(resp2.status(), 401);
}

/// 管理员修改用户密码后，旧密码不能登录，新密码可以。
#[tokio::test]
async fn login_after_admin_password_change() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id =
        create_test_user(&server, &admin_token, "pw_change_user", "old_password").await;

    // 旧密码可以登录
    let resp = user_login(&server, "pw_change_user", "old_password").await;
    assert_eq!(resp.status(), 200);

    // 管理员修改密码
    let update_resp = server
        .put_json_with_token(
            &format!("/api/admin/users/{}", hash_id),
            &json!({"password": "new_password"}),
            &admin_token,
        )
        .await;
    assert_eq!(update_resp.status(), 200);

    // 旧密码不能登录
    let resp2 = user_login(&server, "pw_change_user", "old_password").await;
    assert_eq!(resp2.status(), 401);

    // 新密码可以登录
    let resp3 = user_login(&server, "pw_change_user", "new_password").await;
    assert_eq!(resp3.status(), 200);
}

/// 所有用户保护路由在无认证时都应返回 401。
#[tokio::test]
async fn all_user_routes_require_auth() {
    let server = common::TestServer::spawn().await;

    // PUT /api/users/password
    let resp = server
        .put_json(
            "/api/users/password",
            &json!({"current_password": "x", "new_password": "y"}),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // GET /api/users/subscriptions
    assert_eq!(server.get("/api/users/subscriptions").await.status(), 401);

    // GET /api/articles (user articles list)
    assert_eq!(server.get("/api/users/articles").await.status(), 401);

    // GET /api/articles/:hash_id (user article detail)
    assert_eq!(
        server.get("/api/users/articles/abc123").await.status(),
        401
    );
}
