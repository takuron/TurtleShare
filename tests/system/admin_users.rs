// 管理员用户管理 API 集成测试
//
// 测试 /api/admin/users CRUD 端点的完整行为。
// 包含正常操作、边界条件、错误输入和安全测试。

use super::common;
use serde_json::{json, Value};

// ============================================================
// 辅助函数
// ============================================================

/// 创建一个用户并返回其 hash_id。
async fn create_test_user(
    server: &common::TestServer,
    token: &str,
    username: &str,
) -> String {
    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": username,
                "password": "test_password_123",
                "email": format!("{}@test.com", username),
                "note": format!("Test user {}", username)
            }),
            token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create user {}", username);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

// ============================================================
// 创建用户
// ============================================================

/// 创建用户应返回 201 和正确的用户信息。
#[tokio::test]
async fn create_user_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": "testuser",
                "password": "secure_pass_123",
                "email": "test@example.com",
                "note": "A test user"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["username"], "testuser");
    assert_eq!(body["data"]["email"], "test@example.com");
    assert_eq!(body["data"]["note"], "A test user");

    // hash_id 应该存在且非空
    let hash_id = body["data"]["hash_id"].as_str().unwrap();
    assert!(!hash_id.is_empty());

    // created_at 应该是合理的 Unix 时间戳
    let created_at = body["data"]["created_at"].as_i64().unwrap();
    assert!(created_at > 1_700_000_000); // 2023年之后

    // 响应中不应包含密码哈希
    assert!(body["data"]["password_hash"].is_null());
    assert!(body["data"]["password"].is_null());
    assert!(body["data"]["id"].is_null());
}

/// 创建用户时 email 和 note 可选。
#[tokio::test]
async fn create_user_minimal_fields() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": "minimal_user",
                "password": "pass123"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["username"], "minimal_user");
}

/// 重复用户名应返回 400。
#[tokio::test]
async fn create_user_duplicate_username() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_user(&server, &token, "duplicate_user").await;

    // 再次创建同名用户
    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": "duplicate_user",
                "password": "another_pass"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 未认证创建用户应返回 401。
#[tokio::test]
async fn create_user_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/users",
            &json!({
                "username": "unauthorized_user",
                "password": "pass123"
            }),
        )
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 列出用户
// ============================================================

/// 空数据库应返回空列表。
#[tokio::test]
async fn list_users_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server.get_with_token("/api/admin/users", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let users = body["data"].as_array().unwrap();
    assert!(users.is_empty());
}

/// 创建多个用户后应全部返回。
#[tokio::test]
async fn list_users_returns_all() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_user(&server, &token, "user_a").await;
    create_test_user(&server, &token, "user_b").await;
    create_test_user(&server, &token, "user_c").await;

    let resp = server.get_with_token("/api/admin/users", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 3);

    // 验证返回的用户名
    let usernames: Vec<&str> = users
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(usernames.contains(&"user_a"));
    assert!(usernames.contains(&"user_b"));
    assert!(usernames.contains(&"user_c"));

    // 确保没有泄露密码哈希
    for user in users {
        assert!(user["password_hash"].is_null());
        assert!(user["password"].is_null());
        assert!(user["id"].is_null());
    }
}

// ============================================================
// 获取单个用户
// ============================================================

/// 通过 hash_id 获取用户详情。
#[tokio::test]
async fn get_user_by_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "detail_user").await;

    let resp = server
        .get_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["username"], "detail_user");
    assert_eq!(body["data"]["hash_id"], hash_id);
}

/// 不存在的 hash_id 应返回 404。
#[tokio::test]
async fn get_user_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/users/ZZZZZZ", &token)
        .await;

    // 无效的 hash_id 应返回 400 INVALID_HASH_ID
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 更新用户
// ============================================================

/// 更新用户名应成功。
#[tokio::test]
async fn update_user_username() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "old_name").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/users/{}", hash_id),
            &json!({"username": "new_name"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["username"], "new_name");

    // 验证更改已持久化
    let get_resp = server
        .get_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;
    let get_body: Value = get_resp.json().await.unwrap();
    assert_eq!(get_body["data"]["username"], "new_name");
}

/// 更新 email 和 note。
#[tokio::test]
async fn update_user_email_and_note() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "update_fields").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/users/{}", hash_id),
            &json!({
                "email": "new@email.com",
                "note": "Updated note"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["email"], "new@email.com");
    assert_eq!(body["data"]["note"], "Updated note");
    // 用户名不应改变
    assert_eq!(body["data"]["username"], "update_fields");
}

/// 更新不存在的用户应返回 400（无效 hash_id）。
#[tokio::test]
async fn update_user_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/users/ZZZZZZ",
            &json!({"username": "ghost"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 删除用户
// ============================================================

/// 删除用户应成功。
#[tokio::test]
async fn delete_user_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "to_delete").await;

    let resp = server
        .delete_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["deleted"], true);

    // 验证用户已被删除
    let get_resp = server
        .get_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;
    assert_eq!(get_resp.status(), 404);
}

/// 删除不存在的用户应返回 400（无效 hash_id）。
#[tokio::test]
async fn delete_user_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .delete_with_token("/api/admin/users/ZZZZZZ", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 重复删除同一用户应返回 404。
#[tokio::test]
async fn delete_user_twice() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "delete_twice").await;

    // 第一次删除
    let resp1 = server
        .delete_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;
    assert_eq!(resp1.status(), 200);

    // 第二次删除
    let resp2 = server
        .delete_with_token(&format!("/api/admin/users/{}", hash_id), &token)
        .await;
    assert_eq!(resp2.status(), 404);
}

// ============================================================
// 用户等级查询
// ============================================================

/// 无订阅的用户等级应为 0。
#[tokio::test]
async fn user_tier_defaults_to_zero() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_user(&server, &token, "no_sub_user").await;

    let resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier", hash_id),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 0);
}

// ============================================================
// 安全性测试
// ============================================================

/// XSS 尝试：用户名中包含 HTML/JS 应被原样存储（不执行）。
#[tokio::test]
async fn create_user_xss_in_username() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let xss_name = "<script>alert('xss')</script>";
    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": xss_name,
                "password": "safe_pass"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    // 应原样返回，不做 HTML 转义（API 层面，前端负责转义）
    assert_eq!(body["data"]["username"], xss_name);
}

/// SQL 注入尝试：用户名中包含 SQL 语句不应影响数据库。
#[tokio::test]
async fn create_user_sql_injection_in_fields() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/users",
            &json!({
                "username": "user'; DROP TABLE users; --",
                "password": "pass",
                "email": "' OR '1'='1",
                "note": "Robert'); DROP TABLE users;--"
            }),
            &token,
        )
        .await;

    // 应成功创建（参数化查询保护）
    assert_eq!(resp.status(), 201);

    // 验证 users 表仍然正常工作
    let list_resp = server.get_with_token("/api/admin/users", &token).await;
    assert_eq!(list_resp.status(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    let users = list_body["data"].as_array().unwrap();
    assert_eq!(users.len(), 1); // 只有刚创建的那个用户
}

/// 路径遍历尝试：hash_id 中包含路径字符。
#[tokio::test]
async fn path_traversal_in_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/users/../../etc/passwd", &token)
        .await;

    // 不应返回 200 或泄露文件内容
    let status = resp.status().as_u16();
    assert!(status == 400 || status == 404);
}

/// 所有 CRUD 操作在无认证时都应返回 401。
#[tokio::test]
async fn all_admin_user_routes_require_auth() {
    let server = common::TestServer::spawn().await;

    // GET /api/admin/users
    assert_eq!(server.get("/api/admin/users").await.status(), 401);

    // POST /api/admin/users
    let resp = server
        .post_json(
            "/api/admin/users",
            &json!({"username": "x", "password": "y"}),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // GET /api/admin/users/:id
    assert_eq!(
        server.get("/api/admin/users/abc123").await.status(),
        401
    );

    // PUT /api/admin/users/:id
    let resp = server
        .put_json(
            "/api/admin/users/abc123",
            &json!({"username": "x"}),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // DELETE /api/admin/users/:id
    assert_eq!(
        server.delete("/api/admin/users/abc123").await.status(),
        401
    );

    // GET /api/admin/users/:id/tier
    assert_eq!(
        server.get("/api/admin/users/abc123/tier").await.status(),
        401
    );
}
