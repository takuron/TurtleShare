// publish_at 字段集成测试
//
// 测试 publish_at 字段在管理员创建/更新、用户访问控制、
// 公开端点排序等场景下的正确行为。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 通过管理员 API 创建带自定义 publish_at 的文章，返回完整响应 data。
async fn create_article_with_publish_at(
    server: &common::TestServer,
    token: &str,
    title: &str,
    required_tier: i64,
    is_public: bool,
    publish_at: Option<i64>,
) -> Value {
    let mut body = json!({
        "title": title,
        "content": format!("Content of {}", title),
        "required_tier": required_tier,
        "is_public": is_public,
    });

    if let Some(ts) = publish_at {
        body["publish_at"] = json!(ts);
    }

    let resp = server
        .post_json_with_token("/api/admin/articles", &body, token)
        .await;

    assert_eq!(resp.status(), 201, "Failed to create article {}", title);
    let body: Value = resp.json().await.unwrap();
    body["data"].clone()
}

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
            }),
            admin_token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create user {}", username);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

/// 以用户身份登录，返回 JWT 令牌。
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
) {
    let resp = server
        .post_json_with_token(
            &format!("/api/admin/users/{}/subscriptions", user_hash_id),
            &json!({
                "tier": tier,
                "start_date": start_date,
                "end_date": end_date
            }),
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
// 管理员 - 创建文章时的 publish_at
// ============================================================

/// 不提供 publish_at 时，默认与 created_at 相同。
#[tokio::test]
async fn create_article_publish_at_defaults_to_created_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let data = create_article_with_publish_at(&server, &token, "Default", 0, true, None).await;

    let publish_at = data["publish_at"].as_i64().unwrap();
    let created_at = data["created_at"].as_i64().unwrap();
    assert_eq!(publish_at, created_at);
}

/// 提供正数 publish_at 时，应使用指定值。
#[tokio::test]
async fn create_article_with_explicit_publish_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 指定一个过去的时间作为 publish_at
    let custom_ts: i64 = 1_600_000_000;
    let data =
        create_article_with_publish_at(&server, &token, "Custom", 0, true, Some(custom_ts)).await;

    let publish_at = data["publish_at"].as_i64().unwrap();
    let created_at = data["created_at"].as_i64().unwrap();
    assert_eq!(publish_at, custom_ts);
    // created_at 仍为服务器当前时间，与 publish_at 不同
    assert_ne!(publish_at, created_at);
    assert!(created_at > custom_ts);
}

/// 提供负数 publish_at 时，应回退为 created_at。
#[tokio::test]
async fn create_article_negative_publish_at_defaults_to_created_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let data = create_article_with_publish_at(&server, &token, "Negative", 0, true, Some(-1)).await;

    let publish_at = data["publish_at"].as_i64().unwrap();
    let created_at = data["created_at"].as_i64().unwrap();
    assert_eq!(publish_at, created_at);
}

/// publish_at = 0 应被视为有效值（epoch），不应回退。
#[tokio::test]
async fn create_article_publish_at_zero_is_valid() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let data = create_article_with_publish_at(&server, &token, "Zero", 0, true, Some(0)).await;

    let publish_at = data["publish_at"].as_i64().unwrap();
    assert_eq!(publish_at, 0);
}

// ============================================================
// 管理员 - 更新文章的 publish_at
// ============================================================

/// 更新 publish_at 为指定正数值。
#[tokio::test]
async fn update_article_publish_at_explicit() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let data = create_article_with_publish_at(&server, &token, "Update Me", 0, true, None).await;
    let hash_id = data["hash_id"].as_str().unwrap();
    let original_publish_at = data["publish_at"].as_i64().unwrap();

    let new_ts: i64 = 1_500_000_000;
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"publish_at": new_ts}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let updated_publish_at = body["data"]["publish_at"].as_i64().unwrap();
    assert_eq!(updated_publish_at, new_ts);
    assert_ne!(updated_publish_at, original_publish_at);
}

/// 更新 publish_at 为负数时应重置为 created_at。
#[tokio::test]
async fn update_article_publish_at_negative_resets_to_created_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建文章时指定自定义 publish_at
    let data =
        create_article_with_publish_at(&server, &token, "Reset Me", 0, true, Some(1_500_000_000))
            .await;
    let hash_id = data["hash_id"].as_str().unwrap();
    let created_at = data["created_at"].as_i64().unwrap();

    // 确认 publish_at != created_at
    assert_ne!(data["publish_at"].as_i64().unwrap(), created_at);

    // 用负数重置
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"publish_at": -1}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let reset_publish_at = body["data"]["publish_at"].as_i64().unwrap();
    assert_eq!(reset_publish_at, created_at);
}

/// 不提供 publish_at 的更新不应改变已有的 publish_at。
#[tokio::test]
async fn update_article_without_publish_at_preserves_value() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let custom_ts: i64 = 1_600_000_000;
    let data =
        create_article_with_publish_at(&server, &token, "Preserve", 0, true, Some(custom_ts))
            .await;
    let hash_id = data["hash_id"].as_str().unwrap();

    // 只更新标题，不提供 publish_at
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"title": "Preserved Title"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["publish_at"].as_i64().unwrap(), custom_ts);
}

// ============================================================
// 管理员 - publish_at 排序
// ============================================================

/// 管理员文章列表应按 publish_at 降序排列，即使 created_at 顺序相反。
#[tokio::test]
async fn admin_list_articles_ordered_by_publish_at_not_created_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 先创建的文章给一个很大的 publish_at（排在前面）
    create_article_with_publish_at(
        &server,
        &token,
        "Old Created, New Publish",
        0,
        true,
        Some(9_000_000_000),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 后创建的文章给一个很小的 publish_at（排在后面）
    create_article_with_publish_at(
        &server,
        &token,
        "New Created, Old Publish",
        0,
        true,
        Some(1_000_000_000),
    )
    .await;

    let resp = server.get_with_token("/api/admin/articles", &token).await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    // publish_at 大的（9_000_000_000）应排在前面
    assert_eq!(articles[0]["title"], "Old Created, New Publish");
    assert_eq!(articles[1]["title"], "New Created, Old Publish");

    // 验证排序确实是按 publish_at 降序
    let pa0 = articles[0]["publish_at"].as_i64().unwrap();
    let pa1 = articles[1]["publish_at"].as_i64().unwrap();
    assert!(pa0 > pa1);
}

/// 管理员分页列表同样按 publish_at 降序排列。
#[tokio::test]
async fn admin_paginated_list_ordered_by_publish_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建 3 篇文章，publish_at 逆序于 created_at
    create_article_with_publish_at(&server, &token, "C", 0, true, Some(1_000_000_000)).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_article_with_publish_at(&server, &token, "B", 0, true, Some(2_000_000_000)).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_article_with_publish_at(&server, &token, "A", 0, true, Some(3_000_000_000)).await;

    // 第 1 页，page_size=2
    let resp = server
        .get_with_token("/api/admin/articles/page/1?page_size=2", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let page1 = body["data"].as_array().unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0]["title"], "A"); // publish_at 3B
    assert_eq!(page1[1]["title"], "B"); // publish_at 2B

    // 第 2 页
    let resp = server
        .get_with_token("/api/admin/articles/page/2?page_size=2", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let page2 = body["data"].as_array().unwrap();
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0]["title"], "C"); // publish_at 1B
}

// ============================================================
// 用户端 - publish_at 决定访问控制
// ============================================================

/// 用户订阅覆盖 publish_at 时间时可访问，即使不覆盖 created_at。
/// 证明权限判断使用 publish_at 而非 created_at。
#[tokio::test]
async fn user_access_uses_publish_at_not_created_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id =
        create_test_user(&server, &admin_token, "publish_at_user", "pass123").await;

    // 订阅时段：1_500_000_000 ~ 1_600_000_000
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1_500_000_000,
        1_600_000_000,
    )
    .await;

    let user_token = login_as_user(&server, "publish_at_user", "pass123").await;

    // 文章 A：publish_at 在订阅期内（1_550_000_000），created_at 为当前时间（远超订阅结束）
    // 如果用 created_at 判断，应拒绝；用 publish_at 判断，应允许
    let data_a = create_article_with_publish_at(
        &server,
        &admin_token,
        "Accessible via publish_at",
        2,
        false,
        Some(1_550_000_000),
    )
    .await;
    let hash_a = data_a["hash_id"].as_str().unwrap();

    // 文章 B：publish_at 在订阅期外（1_700_000_000），created_at 也在订阅期外
    let data_b = create_article_with_publish_at(
        &server,
        &admin_token,
        "Inaccessible",
        2,
        false,
        Some(1_700_000_000),
    )
    .await;
    let hash_b = data_b["hash_id"].as_str().unwrap();

    // 文章 A 应可访问（publish_at 在订阅期内）
    let resp_a = server
        .get_with_token(&format!("/api/users/articles/{}", hash_a), &user_token)
        .await;
    assert_eq!(resp_a.status(), 200);

    // 文章 B 应返回 403（publish_at 在订阅期外）
    let resp_b = server
        .get_with_token(&format!("/api/users/articles/{}", hash_b), &user_token)
        .await;
    assert_eq!(resp_b.status(), 403);
}

/// 用户文章列表中的 accessible 字段也基于 publish_at。
#[tokio::test]
async fn user_list_accessible_based_on_publish_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id =
        create_test_user(&server, &admin_token, "list_pa_user", "pass123").await;

    // 订阅：1_500_000_000 ~ 1_600_000_000, tier 2
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1_500_000_000,
        1_600_000_000,
    )
    .await;

    let user_token = login_as_user(&server, "list_pa_user", "pass123").await;

    // 文章 publish_at 在订阅期内 → accessible = true
    create_article_with_publish_at(
        &server,
        &admin_token,
        "In Range",
        2,
        true,
        Some(1_550_000_000),
    )
    .await;

    // 文章 publish_at 在订阅期外 → accessible = false（但公开所以仍可见）
    create_article_with_publish_at(
        &server,
        &admin_token,
        "Out of Range",
        2,
        true,
        Some(1_700_000_000),
    )
    .await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    for article in articles {
        let title = article["title"].as_str().unwrap();
        match title {
            "In Range" => assert_eq!(article["accessible"], true),
            "Out of Range" => assert_eq!(article["accessible"], false),
            other => panic!("Unexpected article: {}", other),
        }
    }
}

/// 用户文章列表按 publish_at 降序排列，即使 created_at 相反。
#[tokio::test]
async fn user_list_ordered_by_publish_at_not_created_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "order_pa_user", "pass123").await;
    let user_token = login_as_user(&server, "order_pa_user", "pass123").await;

    // 先创建的文章给小 publish_at
    create_article_with_publish_at(
        &server,
        &admin_token,
        "First Created",
        0,
        true,
        Some(1_000_000_000),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 后创建的文章给大 publish_at
    create_article_with_publish_at(
        &server,
        &admin_token,
        "Second Created",
        0,
        true,
        Some(9_000_000_000),
    )
    .await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    // publish_at 大的排在前面
    assert_eq!(articles[0]["title"], "Second Created");
    assert_eq!(articles[1]["title"], "First Created");
}

// ============================================================
// 公开端 - publish_at 排序
// ============================================================

/// 公开文章列表按 publish_at 降序排列，即使 created_at 相反。
#[tokio::test]
async fn public_list_ordered_by_publish_at_not_created_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 先创建，大 publish_at
    create_article_with_publish_at(
        &server,
        &admin_token,
        "Pub First",
        0,
        true,
        Some(8_000_000_000),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 后创建，中 publish_at
    create_article_with_publish_at(
        &server,
        &admin_token,
        "Pub Second",
        0,
        true,
        Some(5_000_000_000),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 最后创建，小 publish_at
    create_article_with_publish_at(
        &server,
        &admin_token,
        "Pub Third",
        0,
        true,
        Some(2_000_000_000),
    )
    .await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 3);

    // 按 publish_at 降序排列
    assert_eq!(articles[0]["title"], "Pub First");
    assert_eq!(articles[1]["title"], "Pub Second");
    assert_eq!(articles[2]["title"], "Pub Third");
}

/// 公开文章详情响应包含 publish_at 但不包含 created_at。
#[tokio::test]
async fn public_detail_has_publish_at_no_created_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let custom_ts: i64 = 1_600_000_000;
    let data = create_article_with_publish_at(
        &server,
        &admin_token,
        "Detail Check",
        0,
        true,
        Some(custom_ts),
    )
    .await;
    let hash_id = data["hash_id"].as_str().unwrap();

    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let detail = &body["data"];

    // 应包含 publish_at 且值正确
    assert_eq!(detail["publish_at"].as_i64().unwrap(), custom_ts);

    // 不应包含 created_at
    assert!(detail["created_at"].is_null());
}

/// 用户文章详情响应包含 publish_at 但不包含 created_at。
#[tokio::test]
async fn user_detail_has_publish_at_no_created_at() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "detail_check_user", "pass123").await;
    let user_token = login_as_user(&server, "detail_check_user", "pass123").await;

    let custom_ts: i64 = 1_600_000_000;
    let data = create_article_with_publish_at(
        &server,
        &admin_token,
        "User Detail",
        0,
        true,
        Some(custom_ts),
    )
    .await;
    let hash_id = data["hash_id"].as_str().unwrap();

    let resp = server
        .get_with_token(&format!("/api/users/articles/{}", hash_id), &user_token)
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let detail = &body["data"];

    assert_eq!(detail["publish_at"].as_i64().unwrap(), custom_ts);
    assert!(detail["created_at"].is_null());
}

/// 管理员文章详情响应同时包含 publish_at 和 created_at。
#[tokio::test]
async fn admin_detail_has_both_publish_at_and_created_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let custom_ts: i64 = 1_600_000_000;
    let data = create_article_with_publish_at(
        &server,
        &token,
        "Admin Detail",
        0,
        true,
        Some(custom_ts),
    )
    .await;
    let hash_id = data["hash_id"].as_str().unwrap();

    let resp = server
        .get_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let detail = &body["data"];

    assert_eq!(detail["publish_at"].as_i64().unwrap(), custom_ts);
    assert!(detail["created_at"].is_i64());
    // 两者应不同（自定义了 publish_at）
    assert_ne!(
        detail["publish_at"].as_i64().unwrap(),
        detail["created_at"].as_i64().unwrap()
    );
}
