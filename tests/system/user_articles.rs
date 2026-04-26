// 用户文章 API 集成测试
//
// 测试 GET /api/articles 和 GET /api/articles/:hash_id 端点的完整行为。
// 重点测试基于时间的等级访问控制逻辑。
// 包含正常操作、可见性规则、权限控制和安全测试。

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

/// 通过管理员 API 创建文章，返回 hash_id 和 publish_at。
async fn create_test_article(
    server: &common::TestServer,
    admin_token: &str,
    title: &str,
    required_tier: i64,
    is_public: bool,
) -> (String, i64) {
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": title,
                "content": format!("Content of {}", title),
                "required_tier": required_tier,
                "is_public": is_public,
                "file_links": [{"name": "test.pdf", "url": "https://example.com/files/test.pdf"}]
            }),
            admin_token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create article {}", title);
    let body: Value = resp.json().await.unwrap();
    let hash_id = body["data"]["hash_id"].as_str().unwrap().to_string();
    let publish_at = body["data"]["publish_at"].as_i64().unwrap();
    (hash_id, publish_at)
}

// ============================================================
// 文章列表 - 基本功能
// ============================================================

/// 无文章时用户应看到空列表。
#[tokio::test]
async fn list_articles_empty() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "empty_list_user", "pass123").await;
    let user_token = login_as_user(&server, "empty_list_user", "pass123").await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let articles = body["data"].as_array().unwrap();
    assert!(articles.is_empty());
}

/// tier 0 的公开文章所有用户都可以看到并访问。
#[tokio::test]
async fn list_articles_public_tier0_accessible_to_all() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "free_user", "pass123").await;
    let user_token = login_as_user(&server, "free_user", "pass123").await;

    // 创建 tier 0 的公开文章
    create_test_article(&server, &admin_token, "Free Article", 0, true).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0]["title"], "Free Article");
    assert_eq!(articles[0]["accessible"], true);
}

/// 文章列表响应不应包含 content、is_public 和 file_links。
#[tokio::test]
async fn list_articles_excludes_detail_fields() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "field_user", "pass123").await;
    let user_token = login_as_user(&server, "field_user", "pass123").await;

    create_test_article(&server, &admin_token, "Field Test Article", 0, true).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let article = &body["data"][0];

    // 应包含的字段
    assert!(article["hash_id"].is_string());
    assert!(article["title"].is_string());
    assert!(article["required_tier"].is_i64());
    assert!(!article["accessible"].is_null());
    assert!(article["publish_at"].is_i64());
    assert!(article["updated_at"].is_i64());

    // 不应包含的字段
    assert!(article["content"].is_null());
    assert!(article["is_public"].is_null());
    assert!(article["file_links"].is_null());
    assert!(article["created_at"].is_null());

    // ID 不应泄露
    assert!(article["id"].is_null());
}

/// 文章列表应按 publish_at 降序排列。
#[tokio::test]
async fn list_articles_ordered_by_publish_at_desc() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "order_user", "pass123").await;
    let user_token = login_as_user(&server, "order_user", "pass123").await;

    // 创建三篇文章，间隔足够确保时间戳不同
    create_test_article(&server, &admin_token, "Article A", 0, true).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_test_article(&server, &admin_token, "Article B", 0, true).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_test_article(&server, &admin_token, "Article C", 0, true).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 3);

    // 验证降序排列
    let publish_ats: Vec<i64> = articles
        .iter()
        .map(|a| a["publish_at"].as_i64().unwrap())
        .collect();
    assert!(publish_ats[0] >= publish_ats[1]);
    assert!(publish_ats[1] >= publish_ats[2]);
}

// ============================================================
// 文章列表 - 可见性和访问控制
// ============================================================

/// 公开文章（is_public=true）但等级不足时：可见但 accessible=false。
#[tokio::test]
async fn list_articles_public_insufficient_tier_shows_inaccessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建用户（无订阅，tier 为 0）
    create_test_user(&server, &admin_token, "low_tier_user", "pass123").await;
    let user_token = login_as_user(&server, "low_tier_user", "pass123").await;

    // 创建需要 tier 2 的公开文章
    create_test_article(&server, &admin_token, "Premium Public", 2, true).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0]["title"], "Premium Public");
    assert_eq!(articles[0]["accessible"], false);
}

/// 非公开文章（is_public=false）且等级不足时：不显示在列表中。
#[tokio::test]
async fn list_articles_private_insufficient_tier_hidden() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建用户（无订阅，tier 为 0）
    create_test_user(&server, &admin_token, "hidden_user", "pass123").await;
    let user_token = login_as_user(&server, "hidden_user", "pass123").await;

    // 创建需要 tier 2 的非公开文章
    create_test_article(&server, &admin_token, "Private Premium", 2, false).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    // 非公开且等级不足的文章不应出现
    assert!(articles.is_empty());
}

/// 有足够等级的用户应看到文章且 accessible=true。
#[tokio::test]
async fn list_articles_sufficient_tier_accessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "tier_user", "pass123").await;

    // 先创建覆盖未来时间的订阅（让文章创建时间在订阅期内）
    // 使用非常大的时间范围确保覆盖
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1000000000,
        9999999999,
    )
    .await;

    let user_token = login_as_user(&server, "tier_user", "pass123").await;

    // 创建需要 tier 2 的文章
    create_test_article(&server, &admin_token, "Tier 2 Article", 2, false).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0]["title"], "Tier 2 Article");
    assert_eq!(articles[0]["accessible"], true);
}

/// 混合可见性场景：公开和非公开、不同等级混合。
#[tokio::test]
async fn list_articles_mixed_visibility() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "mix_user", "pass123").await;

    // 给用户 tier 1 的订阅（覆盖很大时间范围）
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        1,
        1000000000,
        9999999999,
    )
    .await;

    let user_token = login_as_user(&server, "mix_user", "pass123").await;

    // 创建多种文章
    create_test_article(&server, &admin_token, "Free Public", 0, true).await; // 可见，accessible=true
    create_test_article(&server, &admin_token, "Tier1 Private", 1, false).await; // 可见，accessible=true
    create_test_article(&server, &admin_token, "Tier2 Public", 2, true).await; // 可见，accessible=false
    create_test_article(&server, &admin_token, "Tier3 Private", 3, false).await; // 不可见

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();

    // 应该能看到 3 篇：Free Public, Tier1 Private, Tier2 Public
    // Tier3 Private 不可见（非公开且等级不足）
    assert_eq!(articles.len(), 3);

    // 按 title 分类检查 accessible 状态
    for article in articles {
        let title = article["title"].as_str().unwrap();
        match title {
            "Free Public" => assert_eq!(article["accessible"], true),
            "Tier1 Private" => assert_eq!(article["accessible"], true),
            "Tier2 Public" => assert_eq!(article["accessible"], false),
            other => panic!("Unexpected article: {}", other),
        }
    }
}

/// 基于时间的访问：订阅过期后创建的文章不可访问。
#[tokio::test]
async fn list_articles_time_based_access() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "time_user", "pass123").await;

    // 给用户一个很早就过期的订阅
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        3,
        1000000000,
        1100000000, // 很早就过期
    )
    .await;

    let user_token = login_as_user(&server, "time_user", "pass123").await;

    // 创建文章（created_at 为当前时间，远晚于订阅结束时间）
    create_test_article(&server, &admin_token, "New Article", 1, true).await;

    let resp = server
        .get_with_token("/api/users/articles", &user_token)
        .await;

    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
    // 虽然用户曾有 tier 3，但在文章创建时已过期，所以不可访问
    assert_eq!(articles[0]["accessible"], false);
}

// ============================================================
// 文章详情 - 成功场景
// ============================================================

/// 有足够等级的用户应能获取文章完整详情。
#[tokio::test]
async fn get_article_detail_accessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "detail_user", "pass123").await;
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        2,
        1000000000,
        9999999999,
    )
    .await;

    let user_token = login_as_user(&server, "detail_user", "pass123").await;

    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Detail Article", 2, false).await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert_eq!(data["hash_id"], article_hash_id);
    assert_eq!(data["title"], "Detail Article");
    assert_eq!(data["content"], "Content of Detail Article");
    assert_eq!(data["required_tier"], 2);

    // 应包含所有详情字段
    assert!(data["publish_at"].is_i64());
    assert!(data["updated_at"].is_i64());
    assert!(data["file_links"].is_array());

    // 不应包含 created_at 和数字 ID
    assert!(data["created_at"].is_null());
    assert!(data["id"].is_null());
}

/// tier 0 文章所有用户都可以获取详情。
#[tokio::test]
async fn get_article_detail_tier0_accessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "tier0_detail_user", "pass123").await;
    let user_token = login_as_user(&server, "tier0_detail_user", "pass123").await;

    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Free Detail", 0, true).await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["title"], "Free Detail");
    assert_eq!(body["data"]["content"], "Content of Free Detail");
}

// ============================================================
// 文章详情 - 拒绝访问
// ============================================================

/// 等级不足的用户应返回 403。
#[tokio::test]
async fn get_article_detail_insufficient_tier() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建无订阅的用户
    create_test_user(&server, &admin_token, "forbidden_user", "pass123").await;
    let user_token = login_as_user(&server, "forbidden_user", "pass123").await;

    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Premium Only", 2, true).await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert!(body["error"]["message"].as_str().unwrap().contains("tier"));
}

/// 公开文章但等级不足也应返回 403（is_public 不授予完整访问权）。
#[tokio::test]
async fn get_article_detail_public_but_insufficient_tier() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "public_no_access", "pass123").await;
    let user_token = login_as_user(&server, "public_no_access", "pass123").await;

    // 创建公开的 tier 2 文章
    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Public Premium", 2, true).await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    // is_public 不授予完整内容访问权限
    assert_eq!(resp.status(), 403);
}

/// 订阅过期后创建的高等级文章应返回 403。
#[tokio::test]
async fn get_article_detail_expired_subscription() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "expired_user", "pass123").await;

    // 很早就过期的 tier 3 订阅
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        3,
        1000000000,
        1100000000,
    )
    .await;

    let user_token = login_as_user(&server, "expired_user", "pass123").await;

    // 文章在订阅过期后创建
    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Post-Expiry", 1, false).await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    // 文章创建时用户已无有效订阅，tier 为 0，不足以访问 tier 1
    assert_eq!(resp.status(), 403);
}

/// 不存在的文章应返回 404。
#[tokio::test]
async fn get_article_detail_not_found() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "notfound_user", "pass123").await;
    let user_token = login_as_user(&server, "notfound_user", "pass123").await;

    // 创建再删除文章，得到合法但不存在的 hash_id
    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Will Delete", 0, true).await;
    server
        .delete_with_token(
            &format!("/api/admin/articles/{}", article_hash_id),
            &admin_token,
        )
        .await;

    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 hash_id 应返回 400。
#[tokio::test]
async fn get_article_detail_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "invalid_hash_user", "pass123").await;
    let user_token = login_as_user(&server, "invalid_hash_user", "pass123").await;

    let resp = server
        .get_with_token("/api/users/articles/INVALID!!!", &user_token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 高级时间-等级访问场景
// ============================================================

/// 多个重叠订阅取最大等级。
#[tokio::test]
async fn article_access_uses_max_overlapping_tier() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let user_hash_id = create_test_user(&server, &admin_token, "overlap_user", "pass123").await;

    // 两个覆盖当前时间的订阅
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        1,
        1000000000,
        9999999999,
    )
    .await;
    create_subscription(
        &server,
        &admin_token,
        &user_hash_id,
        3,
        1000000000,
        9999999999,
    )
    .await;

    let user_token = login_as_user(&server, "overlap_user", "pass123").await;

    // 创建 tier 3 的文章
    let (article_hash_id, _) =
        create_test_article(&server, &admin_token, "Tier3 Article", 3, false).await;

    // 由于重叠订阅取最大值 3，应可访问
    let resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["title"], "Tier3 Article");
}

/// 不同用户看到的文章列表应不同（基于各自等级）。
#[tokio::test]
async fn article_visibility_differs_by_user_tier() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建两个用户，不同等级
    let user_a_hash = create_test_user(&server, &admin_token, "tier_a_user", "pass_a").await;
    let user_b_hash = create_test_user(&server, &admin_token, "tier_b_user", "pass_b").await;

    create_subscription(
        &server,
        &admin_token,
        &user_a_hash,
        1,
        1000000000,
        9999999999,
    )
    .await;
    create_subscription(
        &server,
        &admin_token,
        &user_b_hash,
        3,
        1000000000,
        9999999999,
    )
    .await;

    let token_a = login_as_user(&server, "tier_a_user", "pass_a").await;
    let token_b = login_as_user(&server, "tier_b_user", "pass_b").await;

    // 创建不同等级的私有文章
    create_test_article(&server, &admin_token, "Tier1 Only", 1, false).await;
    create_test_article(&server, &admin_token, "Tier3 Only", 3, false).await;

    // 用户 A (tier 1) 应只看到 Tier1 Only
    let resp_a = server.get_with_token("/api/users/articles", &token_a).await;
    let body_a: Value = resp_a.json().await.unwrap();
    let articles_a = body_a["data"].as_array().unwrap();
    assert_eq!(articles_a.len(), 1);
    assert_eq!(articles_a[0]["title"], "Tier1 Only");

    // 用户 B (tier 3) 应看到两篇
    let resp_b = server.get_with_token("/api/users/articles", &token_b).await;
    let body_b: Value = resp_b.json().await.unwrap();
    let articles_b = body_b["data"].as_array().unwrap();
    assert_eq!(articles_b.len(), 2);
}

/// 文章详情应返回 file_links 字段。
#[tokio::test]
async fn get_article_detail_includes_file_links() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "links_user", "pass123").await;
    let user_token = login_as_user(&server, "links_user", "pass123").await;

    // 创建带 file_links 的 tier 0 文章
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Links Article",
                "content": "Has file links",
                "required_tier": 0,
                "is_public": true,
                "file_links": [
                    {"name": "doc.pdf", "url": "https://example.com/doc.pdf"},
                    {"name": "data.csv", "url": "https://example.com/data.csv"}
                ]
            }),
            &admin_token,
        )
        .await;
    let body: Value = resp.json().await.unwrap();
    let article_hash_id = body["data"]["hash_id"].as_str().unwrap();

    let detail_resp = server
        .get_with_token(
            &format!("/api/users/articles/{}", article_hash_id),
            &user_token,
        )
        .await;

    assert_eq!(detail_resp.status(), 200);
    let detail_body: Value = detail_resp.json().await.unwrap();
    let file_links = detail_body["data"]["file_links"].as_array().unwrap();
    assert_eq!(file_links.len(), 2);
    assert_eq!(file_links[0]["name"], "doc.pdf");
    assert_eq!(file_links[1]["name"], "data.csv");
}

// ============================================================
// 认证测试
// ============================================================

/// 未认证访问文章列表应返回 401。
#[tokio::test]
async fn list_articles_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/users/articles").await;
    assert_eq!(resp.status(), 401);
}

/// 未认证访问文章详情应返回 401。
#[tokio::test]
async fn get_article_detail_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/users/articles/abc123").await;
    assert_eq!(resp.status(), 401);
}

/// 管理员令牌不应能访问用户文章端点。
#[tokio::test]
async fn article_endpoints_reject_admin_token() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 管理员令牌访问用户文章列表
    let resp = server
        .get_with_token("/api/users/articles", &admin_token)
        .await;
    assert_eq!(resp.status(), 403);

    // 管理员令牌访问用户文章详情
    let resp = server
        .get_with_token("/api/users/articles/abc123", &admin_token)
        .await;
    assert_eq!(resp.status(), 403);
}

// ============================================================
// 安全性测试
// ============================================================

/// 路径遍历尝试：hash_id 中包含特殊字符。
#[tokio::test]
async fn article_path_traversal_in_hash_id() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "traversal_user", "pass123").await;
    let user_token = login_as_user(&server, "traversal_user", "pass123").await;

    let resp = server
        .get_with_token("/api/users/articles/../../etc/passwd", &user_token)
        .await;

    let status = resp.status().as_u16();
    assert!(
        status == 400 || status == 404,
        "Expected 400 or 404, got {}",
        status
    );
}
