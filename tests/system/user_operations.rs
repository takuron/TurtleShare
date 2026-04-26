// 用户操作 API 集成测试
//
// 测试 PUT /api/users/password 和 GET /api/users/subscriptions 端点的完整行为。
// 包含正常操作、边界条件、错误输入和安全测试。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 通过管理员 API 创建用户，返回 hash_id。
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

/// 以用户身份登录，返回 JWT 令牌字符串。
async fn login_as_user(server: &common::TestServer, username: &str, password: &str) -> String {
    let resp = server
        .post_json(
            "/api/users/login",
            &json!({
                "username": username,
                "password": password
            }),
        )
        .await;

    assert_eq!(resp.status(), 200, "User login failed for {}", username);
    let body: Value = resp.json().await.unwrap();
    body["data"]["token"].as_str().unwrap().to_string()
}

/// 通过管理员 API 为用户创建订阅。
async fn create_subscription(
    server: &common::TestServer,
    admin_token: &str,
    user_hash_id: &str,
    tier: i64,
    start_date: i64,
    end_date: i64,
    note: Option<&str>,
) {
    let mut body = json!({
        "tier": tier,
        "start_date": start_date,
        "end_date": end_date
    });
    if let Some(n) = note {
        body["note"] = json!(n);
    }

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &body,
            admin_token,
        )
        .await;

    assert_eq!(
        resp.status(),
        201,
        "Failed to create subscription for {}",
        user_hash_id
    );
}

// ============================================================
// 修改密码 - 成功场景
// ============================================================

/// 使用正确的当前密码修改密码应成功。
#[tokio::test]
async fn change_password_success() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "pw_user", "old_password").await;
    let user_token = login_as_user(&server, "pw_user", "old_password").await;

    // 修改密码
    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "old_password",
                "new_password": "new_password_123"
            }),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["message"], "Password changed successfully");
}

/// 修改密码后应能使用新密码登录。
#[tokio::test]
async fn login_with_new_password_after_change() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "pw_login_user", "original_pass").await;
    let user_token = login_as_user(&server, "pw_login_user", "original_pass").await;

    // 修改密码
    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "original_pass",
                "new_password": "changed_pass"
            }),
            &user_token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    // 旧密码不能登录
    let old_resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "pw_login_user", "password": "original_pass"}),
        )
        .await;
    assert_eq!(old_resp.status(), 401);

    // 新密码可以登录
    let new_resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "pw_login_user", "password": "changed_pass"}),
        )
        .await;
    assert_eq!(new_resp.status(), 200);
}

// ============================================================
// 修改密码 - 错误场景
// ============================================================

/// 当前密码错误应返回 401。
#[tokio::test]
async fn change_password_wrong_current() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "pw_wrong_user", "correct_pass").await;
    let user_token = login_as_user(&server, "pw_wrong_user", "correct_pass").await;

    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "wrong_current_pass",
                "new_password": "new_pass_123"
            }),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Current password")
    );
}

/// 新密码为空应返回 400。
#[tokio::test]
async fn change_password_empty_new_password() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "pw_empty_user", "current_pass").await;
    let user_token = login_as_user(&server, "pw_empty_user", "current_pass").await;

    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "current_pass",
                "new_password": ""
            }),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("new_password")
    );
}

/// 新密码仅含空白字符应返回 400。
#[tokio::test]
async fn change_password_whitespace_only_new_password() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "pw_ws_user", "current_pass").await;
    let user_token = login_as_user(&server, "pw_ws_user", "current_pass").await;

    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "current_pass",
                "new_password": "   "
            }),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 未认证修改密码应返回 401。
#[tokio::test]
async fn change_password_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .put_json(
            "/api/users/password",
            &json!({
                "current_password": "old",
                "new_password": "new"
            }),
        )
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 获取自身订阅 - 成功场景
// ============================================================

/// 无订阅的用户应返回空列表。
#[tokio::test]
async fn get_own_subscriptions_empty() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "no_sub_user", "pass123").await;
    let user_token = login_as_user(&server, "no_sub_user", "pass123").await;

    let resp = server
        .get_with_token("/api/users/subscriptions", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let subs = body["data"].as_array().unwrap();
    assert!(subs.is_empty());
}

/// 有订阅的用户应返回正确的订阅信息。
#[tokio::test]
async fn get_own_subscriptions_with_data() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "sub_user", "pass123").await;
    let user_token = login_as_user(&server, "sub_user", "pass123").await;

    // 管理员为用户创建两个订阅
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        1,
        1700000000,
        1710000000,
        Some("Admin note 1"),
    )
    .await;
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1710000000,
        1720000000,
        Some("Admin note 2"),
    )
    .await;

    // 用户获取自身订阅
    let resp = server
        .get_with_token("/api/users/subscriptions", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subs = body["data"].as_array().unwrap();
    assert_eq!(subs.len(), 2);

    // 验证按 start_date 降序排列
    let start_dates: Vec<i64> = subs
        .iter()
        .map(|s| s["start_date"].as_i64().unwrap())
        .collect();
    assert!(start_dates[0] >= start_dates[1]);
}

/// 用户订阅响应不应包含 note 字段（仅管理员可见）。
#[tokio::test]
async fn get_own_subscriptions_excludes_note() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "note_user", "pass123").await;
    let user_token = login_as_user(&server, "note_user", "pass123").await;

    // 创建带 note 的订阅
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        1,
        1700000000,
        1710000000,
        Some("Secret admin note"),
    )
    .await;

    // 用户获取订阅
    let resp = server
        .get_with_token("/api/users/subscriptions", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subs = body["data"].as_array().unwrap();
    assert_eq!(subs.len(), 1);

    // note 字段不应存在
    let sub = &subs[0];
    assert!(
        sub.get("note").is_none() || sub["note"].is_null(),
        "note field should not be present in user subscription response"
    );

    // 验证只包含 tier、start_date、end_date 字段
    assert!(sub["tier"].is_i64());
    assert!(sub["start_date"].is_i64());
    assert!(sub["end_date"].is_i64());
}

/// 用户订阅响应不应包含 hash_id、user_hash_id 等管理员字段。
#[tokio::test]
async fn get_own_subscriptions_excludes_ids() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "id_user", "pass123").await;
    let user_token = login_as_user(&server, "id_user", "pass123").await;

    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1700000000,
        1720000000,
        None,
    )
    .await;

    let resp = server
        .get_with_token("/api/users/subscriptions", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let sub = &body["data"][0];

    // 这些管理员专属字段不应泄露
    assert!(sub["id"].is_null());
    assert!(sub["hash_id"].is_null());
    assert!(sub["user_id"].is_null());
    assert!(sub["user_hash_id"].is_null());
    assert!(sub["created_at"].is_null());
}

/// 用户只能看到自己的订阅，不能看到其他用户的。
#[tokio::test]
async fn get_own_subscriptions_isolated_by_user() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_a_hash = create_test_user(&server, &admin_token, "iso_user_a", "pass_a").await;
    let user_b_hash = create_test_user(&server, &admin_token, "iso_user_b", "pass_b").await;

    // 为两个用户分别创建不同等级的订阅
    create_subscription(
        &server,
        &admin_token,
        &user_a_hash,
        1,
        1700000000,
        1720000000,
        None,
    )
    .await;
    create_subscription(
        &server,
        &admin_token,
        &user_b_hash,
        3,
        1700000000,
        1720000000,
        None,
    )
    .await;

    // 用户 A 应只看到 tier 1
    let token_a = login_as_user(&server, "iso_user_a", "pass_a").await;
    let resp_a = server
        .get_with_token("/api/users/subscriptions", &token_a)
        .await;
    let body_a: Value = resp_a.json().await.unwrap();
    let subs_a = body_a["data"].as_array().unwrap();
    assert_eq!(subs_a.len(), 1);
    assert_eq!(subs_a[0]["tier"], 1);

    // 用户 B 应只看到 tier 3
    let token_b = login_as_user(&server, "iso_user_b", "pass_b").await;
    let resp_b = server
        .get_with_token("/api/users/subscriptions", &token_b)
        .await;
    let body_b: Value = resp_b.json().await.unwrap();
    let subs_b = body_b["data"].as_array().unwrap();
    assert_eq!(subs_b.len(), 1);
    assert_eq!(subs_b[0]["tier"], 3);
}

/// 未认证获取订阅应返回 401。
#[tokio::test]
async fn get_own_subscriptions_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/users/subscriptions").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 安全性测试
// ============================================================

/// SQL 注入尝试：密码字段中包含 SQL 语句不应影响修改密码操作。
#[tokio::test]
async fn change_password_sql_injection() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "sqli_pw_user", "real_pass").await;
    let user_token = login_as_user(&server, "sqli_pw_user", "real_pass").await;

    // SQL 注入尝试在当前密码字段
    let resp = server
        .put_json_with_token(
            "/api/users/password",
            &json!({
                "current_password": "' OR '1'='1' --",
                "new_password": "hacked_pass"
            }),
            &user_token,
        )
        .await;

    // 应返回 401（密码不正确），而不是被注入绕过
    assert_eq!(resp.status(), 401);

    // 原密码仍然有效
    let login_resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "sqli_pw_user", "password": "real_pass"}),
        )
        .await;
    assert_eq!(login_resp.status(), 200);
}
