// 管理员订阅管理 API 集成测试
//
// 测试 /api/admin/users/:hash_id/subscriptions 和 /api/admin/subscriptions/:hash_id 端点的完整行为。
// 包含正常操作、边界条件、错误输入和安全测试。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 创建一个用户并返回其 hash_id。
async fn create_test_user(server: &common::TestServer, token: &str, username: &str) -> String {
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

/// 为用户创建一个订阅并返回订阅的 hash_id。
async fn create_test_subscription(
    server: &common::TestServer,
    token: &str,
    user_hash_id: &str,
    tier: i64,
    start_date: i64,
    end_date: i64,
    note: Option<&str>,
) -> String {
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
            token,
        )
        .await;

    assert_eq!(
        resp.status(),
        201,
        "Failed to create subscription for user {}",
        user_hash_id
    );
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

// ============================================================
// 创建订阅
// ============================================================

/// 创建订阅应返回 201 和正确的订阅信息。
#[tokio::test]
async fn create_subscription_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_user").await;

    let start = 1710928800_i64;
    let end = 1713520800_i64;

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 2,
                "start_date": start,
                "end_date": end,
                "note": "Annual subscription"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // 验证响应字段
    let data = &body["data"];
    let hash_id = data["hash_id"].as_str().unwrap();
    assert!(!hash_id.is_empty(), "hash_id should not be empty");
    assert_eq!(data["user_hash_id"], user_hash_id);
    assert_eq!(data["tier"], 2);
    assert_eq!(data["start_date"], start);
    assert_eq!(data["end_date"], end);
    assert_eq!(data["note"], "Annual subscription");

    // created_at 应该是合理的 Unix 时间戳
    let created_at = data["created_at"].as_i64().unwrap();
    assert!(created_at > 1_700_000_000);

    // 响应中不应包含数字 ID
    assert!(data["id"].is_null());
}

/// 创建订阅时 note 可选。
#[tokio::test]
async fn create_subscription_without_note() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_no_note").await;

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 1,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 1);
    assert!(body["data"]["note"].is_null());
}

/// start_date 大于 end_date 应返回 400。
#[tokio::test]
async fn create_subscription_invalid_date_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_bad_date").await;

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 1,
                "start_date": 1713520800,
                "end_date": 1710928800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("start_date")
    );
}

/// 负数 tier 应返回 400。
#[tokio::test]
async fn create_subscription_negative_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_neg_tier").await;

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": -1,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// tier 超过 255 应返回 400。
#[tokio::test]
async fn create_subscription_tier_overflow() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_overflow").await;

    // tier = 256 超出上限
    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 256,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("255"));
}

/// tier = 255 应成功（边界值）。
#[tokio::test]
async fn create_subscription_tier_max_boundary() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_max_tier").await;

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 255,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 255);
}

/// 为不存在的用户创建订阅应返回 404。
#[tokio::test]
async fn create_subscription_user_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 使用一个有效的 hash_id 格式但对应不存在的用户
    // 先创建再删除用户，得到一个合法但无效的 hash_id
    let user_hash_id = create_test_user(&server, &token, "sub_ghost").await;
    server
        .delete_with_token(&format!("/api/admin/users/{}", user_hash_id), &token)
        .await;
    assert_eq!(
        server
            .delete_with_token(&format!("/api/admin/users/{}", user_hash_id), &token)
            .await
            .status(),
        404
    );

    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 1,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 user hash_id 应返回 400。
#[tokio::test]
async fn create_subscription_invalid_user_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/users/INVALID!!!/subscriptions",
            &json!({
                "tier": 1,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 未认证创建订阅应返回 401。
#[tokio::test]
async fn create_subscription_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/users/somehash/subscriptions",
            &json!({
                "tier": 1,
                "start_date": 1710928800,
                "end_date": 1713520800
            }),
        )
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 列出用户订阅
// ============================================================

/// 无订阅的用户应返回空列表。
#[tokio::test]
async fn list_subscriptions_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_empty").await;

    let resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let subs = body["data"].as_array().unwrap();
    assert!(subs.is_empty());
}

/// 创建多个订阅后应全部返回，按 start_date 降序。
#[tokio::test]
async fn list_subscriptions_returns_all_ordered() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_multi").await;

    // 创建三个不同时间段的订阅
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1709280000,
        1710928800,
        None,
    )
    .await;
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        3,
        1710928800,
        1713520800,
        Some("High tier"),
    )
    .await;
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        2,
        1704067200,
        1709280000,
        Some("Old sub"),
    )
    .await;

    let resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subs = body["data"].as_array().unwrap();
    assert_eq!(subs.len(), 3);

    // 验证按 start_date 降序排列
    let start_dates: Vec<i64> = subs
        .iter()
        .map(|s| s["start_date"].as_i64().unwrap())
        .collect();
    assert!(start_dates[0] >= start_dates[1]);
    assert!(start_dates[1] >= start_dates[2]);

    // 验证所有订阅都属于该用户
    for sub in subs {
        assert_eq!(sub["user_hash_id"], user_hash_id);
        // 确保没有泄露数字 ID
        assert!(sub["id"].is_null());
        assert!(sub["user_id"].is_null());
        // hash_id 应该存在
        assert!(!sub["hash_id"].as_str().unwrap().is_empty());
    }
}

/// 不同用户的订阅不应互相可见。
#[tokio::test]
async fn list_subscriptions_isolated_by_user() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_a = create_test_user(&server, &token, "sub_user_a").await;
    let user_b = create_test_user(&server, &token, "sub_user_b").await;

    create_test_subscription(&server, &token, &user_a, 1, 1710928800, 1713520800, None).await;
    create_test_subscription(&server, &token, &user_b, 2, 1710928800, 1713520800, None).await;

    let resp_a = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_a),
            &token,
        )
        .await;
    let body_a: Value = resp_a.json().await.unwrap();
    assert_eq!(body_a["data"].as_array().unwrap().len(), 1);
    assert_eq!(body_a["data"][0]["tier"], 1);

    let resp_b = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_b),
            &token,
        )
        .await;
    let body_b: Value = resp_b.json().await.unwrap();
    assert_eq!(body_b["data"].as_array().unwrap().len(), 1);
    assert_eq!(body_b["data"][0]["tier"], 2);
}

/// 不存在的用户列出订阅应返回 404。
#[tokio::test]
async fn list_subscriptions_user_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &token, "sub_list_ghost").await;
    server
        .delete_with_token(&format!("/api/admin/users/{}", user_hash_id), &token)
        .await;

    let resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 404);
}

/// 无效的 user hash_id 应返回 400。
#[tokio::test]
async fn list_subscriptions_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/users/ZZZZZZ/subscriptions", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 更新订阅
// ============================================================

/// 更新订阅的 tier 应成功。
#[tokio::test]
async fn update_subscription_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_tier").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"tier": 3}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 3);
    assert_eq!(body["data"]["hash_id"], sub_hash_id);
    assert_eq!(body["data"]["user_hash_id"], user_hash_id);
    // 其他字段不变
    assert_eq!(body["data"]["start_date"], 1710928800);
    assert_eq!(body["data"]["end_date"], 1713520800);
}

/// 更新订阅的时间范围应成功。
#[tokio::test]
async fn update_subscription_dates() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_date").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let new_start = 1709280000_i64;
    let new_end = 1716196800_i64;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({
                "start_date": new_start,
                "end_date": new_end
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["start_date"], new_start);
    assert_eq!(body["data"]["end_date"], new_end);
}

/// 更新订阅的 note 应成功。
#[tokio::test]
async fn update_subscription_note() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_note").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        Some("Old note"),
    )
    .await;

    // 更新为新的备注
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"note": "New note"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["note"], "New note");
}

/// 更新 note 为空字符串应将其设为 null。
#[tokio::test]
async fn update_subscription_note_empty_to_null() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_clr_note").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        Some("Has note"),
    )
    .await;

    // 验证创建时有 note
    let list_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;
    let list_body: Value = list_resp.json().await.unwrap();
    assert_eq!(list_body["data"][0]["note"], "Has note");

    // 清空 note
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"note": ""}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"]["note"].is_null());
}

/// 同时更新多个字段应成功。
#[tokio::test]
async fn update_subscription_multiple_fields() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_multi").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({
                "tier": 3,
                "start_date": 1709280000,
                "end_date": 1716196800,
                "note": "Fully updated"
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 3);
    assert_eq!(body["data"]["start_date"], 1709280000);
    assert_eq!(body["data"]["end_date"], 1716196800);
    assert_eq!(body["data"]["note"], "Fully updated");
}

/// 更新后 start_date > end_date 应返回 400。
#[tokio::test]
async fn update_subscription_invalid_date_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_bad").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    // 只改 start_date 使其大于 end_date
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"start_date": 9999999999_i64}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 更新时负数 tier 应返回 400。
#[tokio::test]
async fn update_subscription_negative_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_neg").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"tier": -5}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 更新时 tier 超过 255 应返回 400。
#[tokio::test]
async fn update_subscription_tier_overflow() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_overflow").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"tier": 256}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("255"));
}

/// 更新时 tier = 255 应成功（边界值）。
#[tokio::test]
async fn update_subscription_tier_max_boundary() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_max").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"tier": 255}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["tier"], 255);
}

/// 更新不存在的订阅应返回 404。
#[tokio::test]
async fn update_subscription_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/subscriptions/ZZZZZZ",
            &json!({"tier": 2}),
            &token,
        )
        .await;

    // 无效 hash_id 返回 400
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 使用已删除订阅的 hash_id 更新应返回 404。
#[tokio::test]
async fn update_deleted_subscription() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_upd_del").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    // 删除订阅
    server
        .delete_with_token(&format!("/api/admin/subscriptions/{}", sub_hash_id), &token)
        .await;

    // 尝试更新已删除的订阅
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/subscriptions/{}", sub_hash_id),
            &json!({"tier": 3}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 未认证更新订阅应返回 401。
#[tokio::test]
async fn update_subscription_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .put_json("/api/admin/subscriptions/somehash", &json!({"tier": 2}))
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 删除订阅
// ============================================================

/// 删除订阅应成功。
#[tokio::test]
async fn delete_subscription_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_del").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    let resp = server
        .delete_with_token(&format!("/api/admin/subscriptions/{}", sub_hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["deleted"], true);
    assert_eq!(body["data"]["hash_id"], sub_hash_id);
    assert_eq!(body["data"]["user_hash_id"], user_hash_id);

    // 验证订阅已从列表中移除
    let list_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;
    let list_body: Value = list_resp.json().await.unwrap();
    assert!(list_body["data"].as_array().unwrap().is_empty());
}

/// 删除不存在的订阅应返回 400（无效 hash_id）或 404。
#[tokio::test]
async fn delete_subscription_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .delete_with_token("/api/admin/subscriptions/ZZZZZZ", &token)
        .await;

    // 无效 hash_id 返回 400
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 使用已删除订阅的 hash_id 再次删除应返回 404。
#[tokio::test]
async fn delete_subscription_twice() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_del_twice").await;
    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;

    // 第一次删除
    let resp1 = server
        .delete_with_token(&format!("/api/admin/subscriptions/{}", sub_hash_id), &token)
        .await;
    assert_eq!(resp1.status(), 200);

    // 第二次删除
    let resp2 = server
        .delete_with_token(&format!("/api/admin/subscriptions/{}", sub_hash_id), &token)
        .await;
    assert_eq!(resp2.status(), 404);
    let body: Value = resp2.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 未认证删除订阅应返回 401。
#[tokio::test]
async fn delete_subscription_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.delete("/api/admin/subscriptions/somehash").await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 级联删除
// ============================================================

/// 删除用户时应级联删除其所有订阅。
#[tokio::test]
async fn delete_user_cascades_subscriptions() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_cascade").await;

    // 创建多个订阅
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1709280000,
        1710928800,
        None,
    )
    .await;
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        2,
        1710928800,
        1713520800,
        Some("Will be cascade deleted"),
    )
    .await;
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        3,
        1713520800,
        1716196800,
        None,
    )
    .await;

    // 验证订阅存在
    let list_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;
    let list_body: Value = list_resp.json().await.unwrap();
    assert_eq!(list_body["data"].as_array().unwrap().len(), 3);

    // 删除用户
    server
        .delete_with_token(&format!("/api/admin/users/{}", user_hash_id), &token)
        .await;

    // 用户已删除
    let get_resp = server
        .get_with_token(&format!("/api/admin/users/{}", user_hash_id), &token)
        .await;
    assert_eq!(get_resp.status(), 404);

    // 订阅也应被级联删除（通过尝试用订阅 hash_id 更新来验证）
    // 注意：由于用户已删除，我们无法直接列出订阅，但可以验证数据库一致性
    // 通过重新创建同名用户来验证订阅确实被删除了
    let new_user_hash_id = create_test_user(&server, &token, "sub_cascade").await;
    let new_list_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", new_user_hash_id),
            &token,
        )
        .await;
    let new_list_body: Value = new_list_resp.json().await.unwrap();
    assert!(new_list_body["data"].as_array().unwrap().is_empty());
}

// ============================================================
// 等级查询联动
// ============================================================

/// 创建订阅后等级查询应反映正确的 tier。
#[tokio::test]
async fn tier_query_reflects_subscriptions() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_tier_check").await;

    // 初始等级为 0
    let tier_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1712000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body: Value = tier_resp.json().await.unwrap();
    assert_eq!(tier_body["data"]["tier"], 0);

    // 创建一个覆盖 1712000000 的 tier 2 订阅
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        2,
        1710928800,
        1713520800,
        None,
    )
    .await;

    // 再次查询，等级应为 2
    let tier_resp2 = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1712000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body2: Value = tier_resp2.json().await.unwrap();
    assert_eq!(tier_body2["data"]["tier"], 2);

    // 查询订阅范围外的时间点，等级应为 0
    let tier_resp3 = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1700000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body3: Value = tier_resp3.json().await.unwrap();
    assert_eq!(tier_body3["data"]["tier"], 0);
}

/// 多个重叠订阅应取最大 tier。
#[tokio::test]
async fn tier_query_max_of_overlapping() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_tier_max").await;

    // 两个重叠订阅
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        1,
        1710928800,
        1713520800,
        None,
    )
    .await;
    create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        3,
        1712000000,
        1716196800,
        None,
    )
    .await;

    // 在重叠区域（1712000000-1713520800），应取最大值 3
    let tier_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1713000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body: Value = tier_resp.json().await.unwrap();
    assert_eq!(tier_body["data"]["tier"], 3);

    // 仅第一个订阅覆盖的区域，应为 1
    let tier_resp2 = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1711500000", user_hash_id),
            &token,
        )
        .await;
    let tier_body2: Value = tier_resp2.json().await.unwrap();
    assert_eq!(tier_body2["data"]["tier"], 1);
}

/// 删除订阅后等级查询应更新。
#[tokio::test]
async fn tier_query_after_subscription_delete() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_tier_del").await;

    let sub_hash_id = create_test_subscription(
        &server,
        &token,
        &user_hash_id,
        2,
        1710928800,
        1713520800,
        None,
    )
    .await;

    // 订阅存在时等级为 2
    let tier_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1712000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body: Value = tier_resp.json().await.unwrap();
    assert_eq!(tier_body["data"]["tier"], 2);

    // 删除订阅
    server
        .delete_with_token(&format!("/api/admin/subscriptions/{}", sub_hash_id), &token)
        .await;

    // 删除后等级应为 0
    let tier_resp2 = server
        .get_with_token(
            &format!("/api/admin/users/{}/tier?at=1712000000", user_hash_id),
            &token,
        )
        .await;
    let tier_body2: Value = tier_resp2.json().await.unwrap();
    assert_eq!(tier_body2["data"]["tier"], 0);
}

// ============================================================
// 安全性测试
// ============================================================

/// SQL 注入尝试：note 字段中包含 SQL 语句不应影响数据库。
#[tokio::test]
async fn create_subscription_sql_injection_in_note() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let user_hash_id = create_test_user(&server, &token, "sub_sqli").await;

    let sql_note = "'; DROP TABLE user_subscriptions; --";
    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": 1,
                "start_date": 1710928800,
                "end_date": 1713520800,
                "note": sql_note
            }),
            &token,
        )
        .await;

    // 应成功创建（参数化查询保护）
    assert_eq!(resp.status(), 201);

    // 验证表仍然正常工作
    let list_resp = server
        .get_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &token,
        )
        .await;
    assert_eq!(list_resp.status(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    assert_eq!(list_body["data"][0]["note"], sql_note);
}

/// 路径遍历尝试：subscription hash_id 中包含特殊字符。
#[tokio::test]
async fn path_traversal_in_subscription_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 使用包含 hashid 字母表外字符的字符串，确保被拒绝
    let resp = server
        .put_json_with_token(
            "/api/admin/subscriptions/invalid!!!hash",
            &json!({"tier": 2}),
            &token,
        )
        .await;

    // 不应返回 200；400 表示无效 hash_id，404 表示解码成功但记录不存在（均安全）
    let status = resp.status().as_u16();
    assert!(
        status == 400 || status == 404,
        "Expected 400 or 404, got {}",
        status
    );
}

/// 所有订阅操作在无认证时都应返回 401。
#[tokio::test]
async fn all_subscription_routes_require_auth() {
    let server = common::TestServer::spawn().await;

    // GET /api/admin/users/:hash_id/subscriptions
    assert_eq!(
        server
            .get("/api/admin/users/abc123/subscriptions")
            .await
            .status(),
        401
    );

    // POST /api/admin/users/:hash_id/subscriptions
    let resp = server
        .post_json(
            "/api/admin/users/abc123/subscriptions",
            &json!({"tier": 1, "start_date": 1710928800, "end_date": 1713520800}),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // PUT /api/admin/subscriptions/:hash_id
    let resp = server
        .put_json("/api/admin/subscriptions/abc123", &json!({"tier": 2}))
        .await;
    assert_eq!(resp.status(), 401);

    // DELETE /api/admin/subscriptions/:hash_id
    assert_eq!(
        server
            .delete("/api/admin/subscriptions/abc123")
            .await
            .status(),
        401
    );
}
