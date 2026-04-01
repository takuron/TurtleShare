// 管理员文章管理 API 集成测试
//
// 测试 /api/admin/articles 和 /api/admin/articles/:hash_id 端点的完整行为。
// 包含正常操作、边界条件、错误输入和安全测试。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 创建一篇测试文章并返回其 hash_id。
async fn create_test_article(
    server: &common::TestServer,
    token: &str,
    title: &str,
) -> String {
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": title,
                "content": format!("Content of {}", title),
                "required_tier": 0,
                "is_public": true
            }),
            token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create article {}", title);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

// ============================================================
// 创建文章
// ============================================================

/// 创建文章应返回 201 和正确的文章信息。
#[tokio::test]
async fn create_article_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Test Article",
                "cover_image": "/files/uuid-123/cover.jpg",
                "content": "# Hello\nThis is a test article.",
                "required_tier": 2,
                "is_public": true,
                "file_links": [
                    {"name": "report.pdf", "url": "https://example.com/files/uuid-123/report.pdf"},
                    {"name": "data.csv", "url": "https://example.com/files/uuid-456/data.csv"}
                ]
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let data = &body["data"];
    let hash_id = data["hash_id"].as_str().unwrap();
    assert!(!hash_id.is_empty(), "hash_id should not be empty");
    assert_eq!(data["title"], "Test Article");
    assert_eq!(data["cover_image"], "/files/uuid-123/cover.jpg");
    assert_eq!(data["content"], "# Hello\nThis is a test article.");
    assert_eq!(data["required_tier"], 2);
    assert_eq!(data["is_public"], true);

    // file_links 应为结构化数组
    let file_links = data["file_links"].as_array().unwrap();
    assert_eq!(file_links.len(), 2);
    assert_eq!(file_links[0]["name"], "report.pdf");
    assert_eq!(file_links[0]["url"], "https://example.com/files/uuid-123/report.pdf");
    assert_eq!(file_links[1]["name"], "data.csv");
    assert_eq!(file_links[1]["url"], "https://example.com/files/uuid-456/data.csv");

    // created_at 和 updated_at 应该是合理的 Unix 时间戳
    let created_at = data["created_at"].as_i64().unwrap();
    let updated_at = data["updated_at"].as_i64().unwrap();
    assert!(created_at > 1_700_000_000);
    assert_eq!(created_at, updated_at);

    // 响应中不应包含数字 ID
    assert!(data["id"].is_null());
}

/// 创建文章时可选字段可以省略。
#[tokio::test]
async fn create_article_minimal() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Minimal Article",
                "content": "Just content",
                "required_tier": 0,
                "is_public": false
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["title"], "Minimal Article");
    assert!(body["data"]["cover_image"].is_null());
    // file_links 省略时应为空数组
    assert_eq!(body["data"]["file_links"], json!([]));
    assert_eq!(body["data"]["is_public"], false);
}

/// 空标题应返回 400。
#[tokio::test]
async fn create_article_empty_title() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "   ",
                "content": "Some content",
                "required_tier": 0,
                "is_public": true
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 负数 required_tier 应返回 400。
#[tokio::test]
async fn create_article_negative_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Bad Tier",
                "content": "Content",
                "required_tier": -1,
                "is_public": true
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// required_tier 超过 255 应返回 400。
#[tokio::test]
async fn create_article_tier_overflow() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Overflow Tier",
                "content": "Content",
                "required_tier": 256,
                "is_public": true
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("255"));
}

/// required_tier = 255 应成功（边界值）。
#[tokio::test]
async fn create_article_tier_max_boundary() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Max Tier",
                "content": "Content",
                "required_tier": 255,
                "is_public": true
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["required_tier"], 255);
}

/// 未认证创建文章应返回 401。
#[tokio::test]
async fn create_article_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .post_json(
            "/api/admin/articles",
            &json!({
                "title": "No Auth",
                "content": "Content",
                "required_tier": 0,
                "is_public": true
            }),
        )
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 列出文章
// ============================================================

/// 无文章时应返回空列表。
#[tokio::test]
async fn list_articles_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/articles", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let articles = body["data"].as_array().unwrap();
    assert!(articles.is_empty());
}

/// 创建多篇文章后应全部返回，按 created_at 降序。
#[tokio::test]
async fn list_articles_returns_all_ordered() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    create_test_article(&server, &token, "Article A").await;
    // 确保时间戳不同
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_test_article(&server, &token, "Article B").await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    create_test_article(&server, &token, "Article C").await;

    let resp = server
        .get_with_token("/api/admin/articles", &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let articles = body["data"].as_array().unwrap();
    assert_eq!(articles.len(), 3);

    // 验证按 created_at 降序排列
    let created_ats: Vec<i64> = articles
        .iter()
        .map(|a| a["created_at"].as_i64().unwrap())
        .collect();
    assert!(created_ats[0] >= created_ats[1]);
    assert!(created_ats[1] >= created_ats[2]);

    // 验证没有泄露数字 ID
    for article in articles {
        assert!(article["id"].is_null());
        assert!(!article["hash_id"].as_str().unwrap().is_empty());
    }
}

/// 未认证列出文章应返回 401。
#[tokio::test]
async fn list_articles_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/articles").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 获取文章详情
// ============================================================

/// 获取文章详情应返回完整信息。
#[tokio::test]
async fn get_article_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_article(&server, &token, "Detail Article").await;

    let resp = server
        .get_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["hash_id"], hash_id);
    assert_eq!(body["data"]["title"], "Detail Article");
    assert!(body["data"]["id"].is_null());
}

/// 不存在的文章应返回 404。
#[tokio::test]
async fn get_article_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建再删除，得到合法但无效的 hash_id
    let hash_id = create_test_article(&server, &token, "Ghost Article").await;
    server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    let resp = server
        .get_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 hash_id 应返回 400。
#[tokio::test]
async fn get_article_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/articles/INVALID!!!", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 更新文章
// ============================================================

/// 更新文章标题应成功。
#[tokio::test]
async fn update_article_title() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Old Title").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"title": "New Title"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["title"], "New Title");
    assert_eq!(body["data"]["hash_id"], hash_id);
}

/// 更新文章内容应成功。
#[tokio::test]
async fn update_article_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Content Update").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"content": "Updated content here"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["content"], "Updated content here");
}

/// 更新 required_tier 和 is_public 应成功。
#[tokio::test]
async fn update_article_tier_and_visibility() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Tier Update").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({
                "required_tier": 3,
                "is_public": false
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["required_tier"], 3);
    assert_eq!(body["data"]["is_public"], false);
}

/// 更新 cover_image 为空字符串应将其设为 null。
#[tokio::test]
async fn update_article_clear_cover_image() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建带封面的文章
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "With Cover",
                "cover_image": "/files/uuid/cover.jpg",
                "content": "Content",
                "required_tier": 0,
                "is_public": true
            }),
            &token,
        )
        .await;
    let body: Value = resp.json().await.unwrap();
    let hash_id = body["data"]["hash_id"].as_str().unwrap();

    // 清空封面
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"cover_image": ""}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"]["cover_image"].is_null());
}

/// 更新 file_links 应成功。
#[tokio::test]
async fn update_article_file_links() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "File Links Update").await;

    // 初始无 file_links，应为空数组
    let get_resp = server
        .get_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;
    let get_body: Value = get_resp.json().await.unwrap();
    assert_eq!(get_body["data"]["file_links"], json!([]));

    // 更新为有文件链接
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({
                "file_links": [
                    {"name": "doc.pdf", "url": "https://example.com/files/abc/doc.pdf"},
                    {"name": "img.png", "url": "https://example.com/files/def/img.png"}
                ]
            }),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let links = body["data"]["file_links"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0]["name"], "doc.pdf");
    assert_eq!(links[0]["url"], "https://example.com/files/abc/doc.pdf");
    assert_eq!(links[1]["name"], "img.png");
    assert_eq!(links[1]["url"], "https://example.com/files/def/img.png");
}

/// 更新 file_links 为空数组应清空。
#[tokio::test]
async fn update_article_clear_file_links() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建带 file_links 的文章
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "Has Links",
                "content": "Content",
                "required_tier": 0,
                "is_public": true,
                "file_links": [{"name": "f.txt", "url": "https://example.com/files/x/f.txt"}]
            }),
            &token,
        )
        .await;
    let body: Value = resp.json().await.unwrap();
    let hash_id = body["data"]["hash_id"].as_str().unwrap();
    assert_eq!(body["data"]["file_links"].as_array().unwrap().len(), 1);

    // 清空 file_links
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"file_links": []}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["file_links"], json!([]));
}

/// 更新应刷新 updated_at 时间戳。
#[tokio::test]
async fn update_article_refreshes_updated_at() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Timestamp Test").await;

    // 获取原始时间戳
    let get_resp = server
        .get_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;
    let get_body: Value = get_resp.json().await.unwrap();
    let original_updated_at = get_body["data"]["updated_at"].as_i64().unwrap();
    let original_created_at = get_body["data"]["created_at"].as_i64().unwrap();

    // 等待一秒确保时间戳不同
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 更新文章
    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"title": "Updated Timestamp"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let new_updated_at = body["data"]["updated_at"].as_i64().unwrap();
    let new_created_at = body["data"]["created_at"].as_i64().unwrap();

    // updated_at 应该更新
    assert!(new_updated_at > original_updated_at);
    // created_at 不应改变
    assert_eq!(new_created_at, original_created_at);
}

/// 更新时空标题应返回 400。
#[tokio::test]
async fn update_article_empty_title() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Title Check").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"title": "  "}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 更新时负数 required_tier 应返回 400。
#[tokio::test]
async fn update_article_negative_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Neg Tier").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"required_tier": -1}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 更新时 required_tier 超过 255 应返回 400。
#[tokio::test]
async fn update_article_tier_overflow() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Overflow Update").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"required_tier": 256}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("255"));
}

/// 更新时 required_tier = 255 应成功（边界值）。
#[tokio::test]
async fn update_article_tier_max_boundary() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "Max Tier Update").await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"required_tier": 255}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["required_tier"], 255);
}

/// 更新不存在的文章应返回 404。
#[tokio::test]
async fn update_article_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_article(&server, &token, "Will Delete").await;
    server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    let resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"title": "Ghost"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 404);
}

/// 未认证更新文章应返回 401。
#[tokio::test]
async fn update_article_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .put_json("/api/admin/articles/somehash", &json!({"title": "No Auth"}))
        .await;

    assert_eq!(resp.status(), 401);
}

// ============================================================
// 删除文章
// ============================================================

/// 删除文章应成功。
#[tokio::test]
async fn delete_article_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let hash_id = create_test_article(&server, &token, "To Delete").await;

    let resp = server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["deleted"], true);
    assert_eq!(body["data"]["hash_id"], hash_id);

    // 验证文章已从列表中移除
    let list_resp = server
        .get_with_token("/api/admin/articles", &token)
        .await;
    let list_body: Value = list_resp.json().await.unwrap();
    assert!(list_body["data"].as_array().unwrap().is_empty());
}

/// 删除不存在的文章应返回 404。
#[tokio::test]
async fn delete_article_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let hash_id = create_test_article(&server, &token, "Delete Twice").await;
    server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    let resp = server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 hash_id 删除应返回 400。
#[tokio::test]
async fn delete_article_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .delete_with_token("/api/admin/articles/INVALID!!!", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 未认证删除文章应返回 401。
#[tokio::test]
async fn delete_article_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.delete("/api/admin/articles/somehash").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 安全性测试
// ============================================================

/// SQL 注入尝试：content 字段中包含 SQL 语句不应影响数据库。
#[tokio::test]
async fn create_article_sql_injection_in_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let sql_content = "'; DROP TABLE articles; --";
    let resp = server
        .post_json_with_token(
            "/api/admin/articles",
            &json!({
                "title": "SQL Injection Test",
                "content": sql_content,
                "required_tier": 0,
                "is_public": true
            }),
            &token,
        )
        .await;

    // 应成功创建（参数化查询保护）
    assert_eq!(resp.status(), 201);

    // 验证表仍然正常工作
    let list_resp = server
        .get_with_token("/api/admin/articles", &token)
        .await;
    assert_eq!(list_resp.status(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    assert_eq!(list_body["data"][0]["content"], sql_content);
}

/// 所有文章操作在无认证时都应返回 401。
#[tokio::test]
async fn all_article_routes_require_auth() {
    let server = common::TestServer::spawn().await;

    // GET /api/admin/articles
    assert_eq!(server.get("/api/admin/articles").await.status(), 401);

    // GET /api/admin/articles/:hash_id
    assert_eq!(
        server.get("/api/admin/articles/abc123").await.status(),
        401
    );

    // POST /api/admin/articles
    let resp = server
        .post_json(
            "/api/admin/articles",
            &json!({
                "title": "No Auth",
                "content": "Content",
                "required_tier": 0,
                "is_public": true
            }),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // PUT /api/admin/articles/:hash_id
    let resp = server
        .put_json(
            "/api/admin/articles/abc123",
            &json!({"title": "No Auth"}),
        )
        .await;
    assert_eq!(resp.status(), 401);

    // DELETE /api/admin/articles/:hash_id
    assert_eq!(
        server.delete("/api/admin/articles/abc123").await.status(),
        401
    );
}
