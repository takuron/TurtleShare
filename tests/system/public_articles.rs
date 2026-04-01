// 公开文章 API 集成测试
//
// 测试 GET /api/public/articles 和 GET /api/public/articles/:hash_id 端点的完整行为。
// 这些是无需鉴权的公开端点，任何人都可以访问。
// 重点测试公开可见性、等级访问控制和安全边界。

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 通过管理员 API 创建文章，返回 hash_id。
async fn create_article(
    server: &common::TestServer,
    admin_token: &str,
    title: &str,
    required_tier: i64,
    is_public: bool,
) -> String {
    create_article_with_options(server, admin_token, title, required_tier, is_public, None, &[])
        .await
}

/// 通过管理员 API 创建文章（带可选参数），返回 hash_id。
async fn create_article_with_options(
    server: &common::TestServer,
    admin_token: &str,
    title: &str,
    required_tier: i64,
    is_public: bool,
    cover_image: Option<&str>,
    file_links: &[(&str, &str)],
) -> String {
    let links: Vec<Value> = file_links
        .iter()
        .map(|(name, url)| json!({"name": name, "url": url}))
        .collect();

    let mut body = json!({
        "title": title,
        "content": format!("Content of {}", title),
        "required_tier": required_tier,
        "is_public": is_public,
        "file_links": links,
    });

    if let Some(img) = cover_image {
        body["cover_image"] = json!(img);
    }

    let resp = server
        .post_json_with_token("/api/admin/articles", &body, admin_token)
        .await;

    assert_eq!(resp.status(), 201, "Failed to create article {}", title);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

// ============================================================
// 公开文章列表 - 基本功能
// ============================================================

/// 无文章时公开端点应返回空列表。
#[tokio::test]
async fn public_list_articles_empty() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/articles").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体结构和空列表
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let data = body["data"].as_array().unwrap();
    assert!(data.is_empty());
}

/// 公开且 tier=0 的文章应在列表中显示为 accessible=true。
#[tokio::test]
async fn public_list_articles_tier0_accessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建一篇公开的免费文章
    create_article(&server, &admin_token, "Free Article", 0, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "Free Article");
    assert_eq!(data[0]["required_tier"], 0);
    assert_eq!(data[0]["accessible"], true);
}

/// 公开但 tier>0 的文章应在列表中显示为 accessible=false。
#[tokio::test]
async fn public_list_articles_tier_gt0_inaccessible() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建一篇公开但需要订阅的文章
    create_article(&server, &admin_token, "Premium Preview", 2, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "Premium Preview");
    assert_eq!(data[0]["required_tier"], 2);
    assert_eq!(data[0]["accessible"], false);
}

/// 非公开文章（is_public=false）不应出现在公开列表中。
#[tokio::test]
async fn public_list_articles_excludes_private() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建一篇私有文章和一篇公开文章
    create_article(&server, &admin_token, "Private Article", 0, false).await;
    create_article(&server, &admin_token, "Public Article", 0, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();

    // 只应该看到公开文章
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "Public Article");
}

/// 公开列表的响应应排除 content、is_public、file_links 字段。
#[tokio::test]
async fn public_list_articles_excludes_detail_fields() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_article(&server, &admin_token, "Test Article", 0, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);

    let article = &data[0];

    // 应包含的字段
    assert!(article["hash_id"].is_string());
    assert!(article["title"].is_string());
    assert!(article["required_tier"].is_number());
    assert!(!article["accessible"].is_null());
    assert!(article["created_at"].is_number());
    assert!(article["updated_at"].is_number());

    // 不应包含的字段
    assert!(article.get("content").is_none() || article["content"].is_null());
    assert!(article.get("is_public").is_none() || article["is_public"].is_null());
    assert!(article.get("file_links").is_none() || article["file_links"].is_null());
}

/// 公开文章列表应按 created_at 降序排列。
#[tokio::test]
async fn public_list_articles_ordered_by_created_at_desc() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 按顺序创建多篇文章（每篇之间等待 1 秒确保 created_at 不同）
    create_article(&server, &admin_token, "First Article", 0, true).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    create_article(&server, &admin_token, "Second Article", 0, true).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    create_article(&server, &admin_token, "Third Article", 0, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 3);

    // 最新的在前（降序）
    assert_eq!(data[0]["title"], "Third Article");
    assert_eq!(data[1]["title"], "Second Article");
    assert_eq!(data[2]["title"], "First Article");
}

// ============================================================
// 公开文章列表 - 混合可见性
// ============================================================

/// 混合的公开/私有和不同 tier 文章应正确过滤和标记。
#[tokio::test]
async fn public_list_articles_mixed_visibility() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建多篇不同配置的文章
    create_article(&server, &admin_token, "Free Public", 0, true).await;
    create_article(&server, &admin_token, "Premium Public", 3, true).await;
    create_article(&server, &admin_token, "Free Private", 0, false).await;
    create_article(&server, &admin_token, "Premium Private", 2, false).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();

    // 只有 is_public=true 的文章出现
    assert_eq!(data.len(), 2);
    let titles: Vec<&str> = data.iter().map(|a| a["title"].as_str().unwrap()).collect();
    assert!(titles.contains(&"Free Public"));
    assert!(titles.contains(&"Premium Public"));
    assert!(!titles.contains(&"Free Private"));
    assert!(!titles.contains(&"Premium Private"));

    // 验证 accessible 标记
    for article in data {
        let title = article["title"].as_str().unwrap();
        match title {
            "Free Public" => assert_eq!(article["accessible"], true),
            "Premium Public" => assert_eq!(article["accessible"], false),
            _ => panic!("Unexpected article: {}", title),
        }
    }
}

/// 列表中的 cover_image 字段应正确返回（包括 null）。
#[tokio::test]
async fn public_list_articles_cover_image_field() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 一篇有封面图，一篇没有
    create_article_with_options(
        &server,
        &admin_token,
        "With Cover",
        0,
        true,
        Some("/files/uuid/cover.jpg"),
        &[],
    )
    .await;
    create_article(&server, &admin_token, "No Cover", 0, true).await;

    let resp = server.get("/api/public/articles").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    for article in data {
        let title = article["title"].as_str().unwrap();
        match title {
            "With Cover" => {
                assert_eq!(article["cover_image"], "/files/uuid/cover.jpg");
            }
            "No Cover" => {
                assert!(article["cover_image"].is_null());
            }
            _ => panic!("Unexpected article: {}", title),
        }
    }
}

// ============================================================
// 公开文章详情 - 成功场景
// ============================================================

/// 公开且 tier=0 的文章应返回完整内容。
#[tokio::test]
async fn public_get_article_detail_tier0_success() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article_with_options(
        &server,
        &admin_token,
        "Free Full Article",
        0,
        true,
        Some("/files/uuid/cover.jpg"),
        &[("doc.pdf", "https://example.com/files/uuid/doc.pdf")],
    )
    .await;

    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let data = &body["data"];

    // 验证所有字段
    assert_eq!(data["hash_id"], hash_id);
    assert_eq!(data["title"], "Free Full Article");
    assert_eq!(data["cover_image"], "/files/uuid/cover.jpg");
    assert_eq!(data["content"], "Content of Free Full Article");
    assert_eq!(data["required_tier"], 0);
    assert_eq!(data["is_public"], true);
    assert!(data["created_at"].is_number());
    assert!(data["updated_at"].is_number());

    // 验证 file_links
    let file_links = data["file_links"].as_array().unwrap();
    assert_eq!(file_links.len(), 1);
    assert_eq!(file_links[0]["name"], "doc.pdf");
    assert_eq!(file_links[0]["url"], "https://example.com/files/uuid/doc.pdf");
}

/// 公开 tier=0 文章无 file_links 时应返回空数组。
#[tokio::test]
async fn public_get_article_detail_empty_file_links() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "No Links Article", 0, true).await;

    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let file_links = body["data"]["file_links"].as_array().unwrap();
    assert!(file_links.is_empty());
}

// ============================================================
// 公开文章详情 - 拒绝场景
// ============================================================

/// 公开但 required_tier > 0 的文章详情应返回 403 Forbidden。
#[tokio::test]
async fn public_get_article_detail_tier_gt0_forbidden() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "Premium Article", 2, true).await;

    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 403);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
}

/// 非公开文章（is_public=false）通过公开端点访问应返回 404。
#[tokio::test]
async fn public_get_article_detail_private_returns_404() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "Private Article", 0, false).await;

    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 404);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 不存在的 hash_id 应返回 400 或 404。
#[tokio::test]
async fn public_get_article_detail_nonexistent_returns_error() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/articles/zzzzzz").await;
    let status = resp.status().as_u16();
    // 无法解码的 hash_id 返回 400，有效但不存在的返回 404
    assert!(
        status == 400 || status == 404,
        "Expected 400 or 404, got {}",
        status
    );

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
}

/// 已删除的文章通过公开端点访问应返回 404。
#[tokio::test]
async fn public_get_article_detail_deleted_returns_404() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建并删除文章
    let hash_id = create_article(&server, &admin_token, "Doomed Article", 0, true).await;
    let del_resp = server
        .delete_with_token(&format!("/api/admin/articles/{}", hash_id), &admin_token)
        .await;
    assert_eq!(del_resp.status(), 200);

    // 通过公开端点访问已删除的文章
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 404);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 即使 is_public=true，required_tier > 0 的文章也应返回 403 而不是 200。
#[tokio::test]
async fn public_get_article_detail_public_but_paid_forbidden() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建多个不同 tier 的公开文章
    let hash_tier1 = create_article(&server, &admin_token, "Tier 1 Public", 1, true).await;
    let hash_tier5 = create_article(&server, &admin_token, "Tier 5 Public", 5, true).await;

    // tier 1 文章 - 403
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_tier1))
        .await;
    assert_eq!(resp.status(), 403);

    // tier 5 文章 - 403
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_tier5))
        .await;
    assert_eq!(resp.status(), 403);
}

// ============================================================
// 公开端点 - 无需鉴权验证
// ============================================================

/// 公开端点无需任何鉴权即可访问。
#[tokio::test]
async fn public_endpoints_require_no_auth() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 创建测试文章
    let hash_id = create_article(&server, &admin_token, "Public Test", 0, true).await;

    // 直接无 token 访问列表和详情
    let list_resp = server.get("/api/public/articles").await;
    assert_eq!(list_resp.status(), 200);

    let detail_resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(detail_resp.status(), 200);
}

/// 即使带有 token 也应正常工作（公开端点忽略鉴权头）。
#[tokio::test]
async fn public_endpoints_work_with_token_present() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "Token Test", 0, true).await;

    // 带 admin token 访问公开端点
    let list_resp = server
        .get_with_token("/api/public/articles", &admin_token)
        .await;
    assert_eq!(list_resp.status(), 200);

    let detail_resp = server
        .get_with_token(
            &format!("/api/public/articles/{}", hash_id),
            &admin_token,
        )
        .await;
    assert_eq!(detail_resp.status(), 200);
}

// ============================================================
// 公开端点 - 文章更新后行为
// ============================================================

/// 文章从公开改为私有后，不应出现在公开列表中。
#[tokio::test]
async fn public_list_articles_after_made_private() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "Now Private", 0, true).await;

    // 确认初始可见
    let resp = server.get("/api/public/articles").await;
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // 将文章设为私有
    let update_resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"is_public": false}),
            &admin_token,
        )
        .await;
    assert_eq!(update_resp.status(), 200);

    // 公开列表应为空
    let resp = server.get("/api/public/articles").await;
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // 详情也应返回 404
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 404);
}

/// 文章 tier 从 0 改为 2 后，公开详情应返回 403。
#[tokio::test]
async fn public_get_article_detail_after_tier_changed() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    let hash_id = create_article(&server, &admin_token, "Tier Change", 0, true).await;

    // 初始可访问
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 200);

    // 提升 tier
    let update_resp = server
        .put_json_with_token(
            &format!("/api/admin/articles/{}", hash_id),
            &json!({"required_tier": 2}),
            &admin_token,
        )
        .await;
    assert_eq!(update_resp.status(), 200);

    // 现在应该返回 403
    let resp = server
        .get(&format!("/api/public/articles/{}", hash_id))
        .await;
    assert_eq!(resp.status(), 403);

    // 列表中仍可见但 accessible=false
    let list_resp = server.get("/api/public/articles").await;
    let body: Value = list_resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["accessible"], false);
}

// ============================================================
// 安全测试
// ============================================================

/// 路径遍历攻击 hash_id 不应导致信息泄露。
#[tokio::test]
async fn public_article_path_traversal_in_hash_id() {
    let server = common::TestServer::spawn().await;

    let malicious_ids = [
        "../../etc/passwd",
        "../admin/articles",
        "%2e%2e%2f",
        "..\\..\\",
    ];

    for id in &malicious_ids {
        let resp = server
            .get(&format!("/api/public/articles/{}", id))
            .await;
        let status = resp.status().as_u16();
        // 应返回 400 或 404，而不是 200 或 500
        assert!(
            status == 400 || status == 404,
            "Path traversal '{}' returned unexpected status {}",
            id,
            status
        );
    }
}

/// 无效的 hash_id 格式应返回合理的错误。
#[tokio::test]
async fn public_article_invalid_hash_id() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/articles/!!!invalid!!!").await;
    let status = resp.status().as_u16();
    // 应返回 400 或 404
    assert!(
        status == 400 || status == 404,
        "Invalid hash_id returned unexpected status {}",
        status
    );
}
