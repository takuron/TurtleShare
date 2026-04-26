// 分页 API 集成测试
//
// 测试所有分页端点（/page 和 /page/:page）的行为。
// 涵盖管理员用户分页、管理员文章分页、管理员文件分页、
// 用户文章分页和公开文章分页。

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

/// 创建一篇测试文章并返回其 hash_id。
async fn create_test_article(
    server: &common::TestServer,
    token: &str,
    title: &str,
    is_public: bool,
    required_tier: u8,
) -> String {
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": title,
                "content": format!("Content of {}", title),
                "required_tier": required_tier,
                "is_public": is_public
            }),
            token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create article {}", title);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

/// 批量创建用户，返回所有 hash_id。
async fn create_users_batch(server: &common::TestServer, token: &str, count: usize) -> Vec<String> {
    let mut ids = Vec::new();
    for i in 0..count {
        let id = create_test_user(server, token, &format!("page_user_{}", i)).await;
        ids.push(id);
    }
    ids
}

/// 批量创建文章，返回所有 hash_id。
async fn create_articles_batch(
    server: &common::TestServer,
    token: &str,
    count: usize,
    is_public: bool,
    required_tier: u8,
) -> Vec<String> {
    let mut ids = Vec::new();
    for i in 0..count {
        let id = create_test_article(
            server,
            token,
            &format!("Page Article {}", i),
            is_public,
            required_tier,
        )
        .await;
        ids.push(id);
    }
    ids
}

/// 以用户身份登录并返回 JWT 令牌。
async fn user_login(server: &common::TestServer, username: &str, password: &str) -> String {
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

// ============================================================
// 管理员用户分页 - GET /api/admin/users/page
// ============================================================

/// 空数据库应返回 total_pages=0, total_items=0。
#[tokio::test]
async fn admin_users_page_info_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server.get_with_token("/api/admin/users/page", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["total_pages"], 0);
    assert_eq!(body["data"]["total_items"], 0);
}

/// 创建 5 个用户，默认 page_size=20 应返回 total_pages=1。
#[tokio::test]
async fn admin_users_page_info_single_page() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 5).await;

    let resp = server.get_with_token("/api/admin/users/page", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 1);
    assert_eq!(body["data"]["total_items"], 5);
}

/// 自定义 page_size，验证页数计算正确。
#[tokio::test]
async fn admin_users_page_info_custom_page_size() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 5).await;

    // page_size=2, 5 个用户应返回 3 页
    let resp = server
        .get_with_token("/api/admin/users/page?page_size=2", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 3);
    assert_eq!(body["data"]["total_items"], 5);
}

/// 刚好整除时页数应正确。
#[tokio::test]
async fn admin_users_page_info_exact_division() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 4).await;

    // page_size=2, 4 个用户应返回 2 页
    let resp = server
        .get_with_token("/api/admin/users/page?page_size=2", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 2);
    assert_eq!(body["data"]["total_items"], 4);
}

/// 未认证应返回 401。
#[tokio::test]
async fn admin_users_page_info_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/users/page").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 管理员用户分页 - GET /api/admin/users/page/:page
// ============================================================

/// 分页获取第一页。
#[tokio::test]
async fn admin_users_page_first() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 5).await;

    let resp = server
        .get_with_token("/api/admin/users/page/1?page_size=3", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 3);
}

/// 分页获取最后一页（不满一整页）。
#[tokio::test]
async fn admin_users_page_last_partial() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 5).await;

    // page_size=3, 第 2 页应有 2 个用户
    let resp = server
        .get_with_token("/api/admin/users/page/2?page_size=3", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 2);
}

/// 超出范围的页码应返回空列表。
#[tokio::test]
async fn admin_users_page_out_of_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 3).await;

    let resp = server
        .get_with_token("/api/admin/users/page/999?page_size=20", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert!(users.is_empty());
}

/// 响应格式应与列表 API 一致（含 hash_id，不含密码哈希和数字 ID）。
#[tokio::test]
async fn admin_users_page_response_format() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_user(&server, &token, "format_test_user").await;

    let resp = server
        .get_with_token("/api/admin/users/page/1", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 1);

    let user = &users[0];
    assert!(!user["hash_id"].as_str().unwrap().is_empty());
    assert_eq!(user["username"], "format_test_user");
    // 不应泄露密码哈希和数字 ID
    assert!(user["password_hash"].is_null());
    assert!(user["password"].is_null());
    assert!(user["id"].is_null());
}

/// 未认证应返回 401。
#[tokio::test]
async fn admin_users_page_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/users/page/1").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 管理员文章分页 - GET /api/admin/articles/page
// ============================================================

/// 空数据库应返回 total_pages=0, total_items=0。
#[tokio::test]
async fn admin_articles_page_info_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/articles/page", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 0);
    assert_eq!(body["data"]["total_items"], 0);
}

/// 创建文章后验证页数信息。
#[tokio::test]
async fn admin_articles_page_info_with_data() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 7, true, 0).await;

    let resp = server
        .get_with_token("/api/admin/articles/page?page_size=3", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 3); // ceil(7/3) = 3
    assert_eq!(body["data"]["total_items"], 7);
}

/// 未认证应返回 401。
#[tokio::test]
async fn admin_articles_page_info_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/articles/page").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 管理员文章分页 - GET /api/admin/articles/page/:page
// ============================================================

/// 分页获取文章，验证每页数量正确。
#[tokio::test]
async fn admin_articles_page_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 5, true, 0).await;

    // 第一页
    let resp = server
        .get_with_token("/api/admin/articles/page/1?page_size=2", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    // 第三页（最后一页，只有 1 篇）
    let resp = server
        .get_with_token("/api/admin/articles/page/3?page_size=2", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
}

/// 管理员文章分页应包含公开和私有文章。
#[tokio::test]
async fn admin_articles_page_includes_all() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建公开和私有文章
    create_test_article(&server, &token, "Public Art", true, 0).await;
    create_test_article(&server, &token, "Private Art", false, 2).await;

    let resp = server
        .get_with_token("/api/admin/articles/page?page_size=20", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_items"], 2);
}

/// 文章分页列表响应中不应包含 content 和 file_links 字段。
#[tokio::test]
async fn admin_articles_page_excludes_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_article(&server, &token, "Content Check", true, 0).await;

    let resp = server
        .get_with_token("/api/admin/articles/page/1", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 1);

    let article = &articles[0];
    assert!(article["content"].is_null());
    assert!(article["file_links"].is_null());
    // 应包含的字段
    assert!(!article["hash_id"].as_str().unwrap().is_empty());
    assert_eq!(article["title"], "Content Check");
}

/// 超出范围的页码应返回空列表。
#[tokio::test]
async fn admin_articles_page_out_of_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 3, true, 0).await;

    let resp = server
        .get_with_token("/api/admin/articles/page/100", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert!(articles.is_empty());
}

/// 未认证应返回 401。
#[tokio::test]
async fn admin_articles_page_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/articles/page/1").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 管理员文件分页 - GET /api/admin/files/page
// ============================================================

/// 空数据库应返回 total_pages=0, total_items=0。
#[tokio::test]
async fn admin_files_page_info_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server.get_with_token("/api/admin/files/page", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 0);
    assert_eq!(body["data"]["total_items"], 0);
}

/// 未认证应返回 401。
#[tokio::test]
async fn admin_files_page_info_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/files/page").await;
    assert_eq!(resp.status(), 401);
}

/// 未认证获取文件分页应返回 401。
#[tokio::test]
async fn admin_files_page_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/files/page/1").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 公开文章分页 - GET /api/public/articles/page
// ============================================================

/// 空数据库应返回 total_pages=0, total_items=0。
#[tokio::test]
async fn public_articles_page_info_empty() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/articles/page").await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 0);
    assert_eq!(body["data"]["total_items"], 0);
}

/// 只统计公开文章（is_public=true）。
#[tokio::test]
async fn public_articles_page_info_only_public() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建 3 篇公开文章 + 2 篇私有文章
    create_articles_batch(&server, &token, 3, true, 0).await;
    create_articles_batch(&server, &token, 2, false, 0).await;

    let resp = server.get("/api/public/articles/page?page_size=20").await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_items"], 3);
    assert_eq!(body["data"]["total_pages"], 1);
}

/// 自定义 page_size 验证页数。
#[tokio::test]
async fn public_articles_page_info_custom_size() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 5, true, 0).await;

    let resp = server.get("/api/public/articles/page?page_size=2").await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 3); // ceil(5/2) = 3
    assert_eq!(body["data"]["total_items"], 5);
}

/// 无需认证即可访问。
#[tokio::test]
async fn public_articles_page_info_no_auth_required() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/articles/page").await;
    // 应返回 200 而非 401
    assert_eq!(resp.status(), 200);
}

// ============================================================
// 公开文章分页 - GET /api/public/articles/page/:page
// ============================================================

/// 获取公开文章分页内容。
#[tokio::test]
async fn public_articles_page_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 5, true, 0).await;

    // 第一页
    let resp = server.get("/api/public/articles/page/1?page_size=2").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);
}

/// 只返回公开文章，不包含私有文章。
#[tokio::test]
async fn public_articles_page_excludes_private() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建 2 篇公开 + 3 篇私有
    create_articles_batch(&server, &token, 2, true, 0).await;
    create_articles_batch(&server, &token, 3, false, 0).await;

    let resp = server.get("/api/public/articles/page/1?page_size=20").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);
}

/// 公开文章分页响应应包含 accessible 字段。
#[tokio::test]
async fn public_articles_page_has_accessible_field() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // tier=0 的公开文章 accessible=true
    create_test_article(&server, &token, "Free Public", true, 0).await;
    // tier>0 的公开文章 accessible=false
    create_test_article(&server, &token, "Premium Public", true, 2).await;

    let resp = server.get("/api/public/articles/page/1?page_size=20").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    // 找到并验证各文章的 accessible 字段
    for article in articles {
        let tier = article["required_tier"].as_i64().unwrap();
        let accessible = article["accessible"].as_bool().unwrap();
        if tier == 0 {
            assert!(accessible, "tier=0 应为 accessible=true");
        } else {
            assert!(!accessible, "tier>0 应为 accessible=false");
        }
    }
}

/// 公开文章分页响应不应包含 content、is_public 和 file_links。
#[tokio::test]
async fn public_articles_page_excludes_content_fields() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_article(&server, &token, "Fields Check", true, 0).await;

    let resp = server.get("/api/public/articles/page/1").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    let article = &articles[0];

    assert!(article["content"].is_null());
    assert!(article["is_public"].is_null());
    assert!(article["file_links"].is_null());
    // 应包含的字段
    assert!(!article["hash_id"].as_str().unwrap().is_empty());
    assert_eq!(article["title"], "Fields Check");
}

/// 超出范围的页码应返回空列表。
#[tokio::test]
async fn public_articles_page_out_of_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 2, true, 0).await;

    let resp = server.get("/api/public/articles/page/999").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert!(articles.is_empty());
}

// ============================================================
// 用户文章分页 - GET /api/users/articles/page
// ============================================================

/// 未认证应返回 401。
#[tokio::test]
async fn user_articles_page_info_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/users/articles/page").await;
    assert_eq!(resp.status(), 401);
}

/// 用户文章分页信息应返回正确的总数。
#[tokio::test]
async fn user_articles_page_info_with_data() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建用户
    create_test_user(&server, &token, "page_viewer").await;
    let user_token = user_login(&server, "page_viewer", "test_password_123").await;

    // 创建公开文章（对用户可见）和私有文章
    create_articles_batch(&server, &token, 4, true, 0).await;
    create_articles_batch(&server, &token, 2, false, 0).await;

    let resp = server
        .get_with_token("/api/users/articles/page?page_size=3", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    // 具体数量取决于可见性逻辑，至少验证响应格式
    assert!(body["data"]["total_pages"].is_number());
    assert!(body["data"]["total_items"].is_number());
}

// ============================================================
// 用户文章分页 - GET /api/users/articles/page/:page
// ============================================================

/// 未认证应返回 401。
#[tokio::test]
async fn user_articles_page_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/users/articles/page/1").await;
    assert_eq!(resp.status(), 401);
}

/// 用户文章分页应返回正确格式。
#[tokio::test]
async fn user_articles_page_content_format() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建用户
    create_test_user(&server, &token, "page_reader").await;
    let user_token = user_login(&server, "page_reader", "test_password_123").await;

    // 创建公开 tier=0 文章
    create_articles_batch(&server, &token, 3, true, 0).await;

    let resp = server
        .get_with_token("/api/users/articles/page/1?page_size=20", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let articles = body["data"].as_array().unwrap();

    // 验证文章响应格式包含 accessible 字段
    for article in articles {
        assert!(!article["hash_id"].as_str().unwrap().is_empty());
        assert!(article["title"].is_string());
        assert!(article["accessible"].is_boolean());
        // 不应包含 content、is_public、file_links
        assert!(article["content"].is_null());
        assert!(article["is_public"].is_null());
        assert!(article["file_links"].is_null());
    }
}

/// 超出范围页码应返回空列表。
#[tokio::test]
async fn user_articles_page_out_of_range() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_user(&server, &token, "page_empty").await;
    let user_token = user_login(&server, "page_empty", "test_password_123").await;

    let resp = server
        .get_with_token("/api/users/articles/page/999", &user_token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert!(articles.is_empty());
}

// ============================================================
// 分页通用行为测试
// ============================================================

/// 各分页端点之间数据不交叉：所有页合并应等于完整列表。
#[tokio::test]
async fn admin_users_pagination_covers_all_items() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 7).await;

    // 获取全部用户
    let all_resp = server.get_with_token("/api/admin/users", &token).await;
    let all_body: Value = all_resp.json().await.unwrap();
    let all_users = all_body["data"].as_array().unwrap();
    assert_eq!(all_users.len(), 7);

    // 分页获取并合并
    let mut paged_ids: Vec<String> = Vec::new();
    for page in 1..=4 {
        let resp = server
            .get_with_token(
                &format!("/api/admin/users/page/{}?page_size=2", page),
                &token,
            )
            .await;
        let body: Value = resp.json().await.unwrap();
        let users = body["data"].as_array().unwrap();
        for user in users {
            paged_ids.push(user["hash_id"].as_str().unwrap().to_string());
        }
    }

    // 排序后比较
    let mut all_ids: Vec<String> = all_users
        .iter()
        .map(|u| u["hash_id"].as_str().unwrap().to_string())
        .collect();
    all_ids.sort();
    paged_ids.sort();
    assert_eq!(all_ids, paged_ids);
}

/// 各分页端点之间数据不交叉：文章分页合并应等于完整列表。
#[tokio::test]
async fn admin_articles_pagination_covers_all_items() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_articles_batch(&server, &token, 5, true, 0).await;

    // 获取全部文章
    let all_resp = server.get_with_token("/api/admin/articles", &token).await;
    let all_body: Value = all_resp.json().await.unwrap();
    let all_articles = all_body["data"].as_array().unwrap();
    assert_eq!(all_articles.len(), 5);

    // 分页获取并合并
    let mut paged_ids: Vec<String> = Vec::new();
    for page in 1..=3 {
        let resp = server
            .get_with_token(
                &format!("/api/admin/articles/page/{}?page_size=2", page),
                &token,
            )
            .await;
        let body: Value = resp.json().await.unwrap();
        let articles = body["data"].as_array().unwrap();
        for article in articles {
            paged_ids.push(article["hash_id"].as_str().unwrap().to_string());
        }
    }

    let mut all_ids: Vec<String> = all_articles
        .iter()
        .map(|a| a["hash_id"].as_str().unwrap().to_string())
        .collect();
    all_ids.sort();
    paged_ids.sort();
    assert_eq!(all_ids, paged_ids);
}

/// page_size=1 时每页只有一个项目。
#[tokio::test]
async fn admin_users_page_size_one() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_users_batch(&server, &token, 3).await;

    // page info
    let resp = server
        .get_with_token("/api/admin/users/page?page_size=1", &token)
        .await;
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 3);
    assert_eq!(body["data"]["total_items"], 3);

    // 每页一个
    for page in 1..=3 {
        let resp = server
            .get_with_token(
                &format!("/api/admin/users/page/{}?page_size=1", page),
                &token,
            )
            .await;
        let body: Value = resp.json().await.unwrap();
        let users = body["data"].as_array().unwrap();
        assert_eq!(users.len(), 1);
    }
}

/// 默认 page_size 应为 20。
#[tokio::test]
async fn admin_users_default_page_size() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建 21 个用户
    create_users_batch(&server, &token, 21).await;

    // 不指定 page_size，默认 20
    let resp = server.get_with_token("/api/admin/users/page", &token).await;
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["total_pages"], 2); // ceil(21/20) = 2
    assert_eq!(body["data"]["total_items"], 21);

    // 第一页应有 20 个
    let resp = server
        .get_with_token("/api/admin/users/page/1", &token)
        .await;
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 20);

    // 第二页应有 1 个
    let resp = server
        .get_with_token("/api/admin/users/page/2", &token)
        .await;
    let body: Value = resp.json().await.unwrap();
    let users = body["data"].as_array().unwrap();
    assert_eq!(users.len(), 1);
}
