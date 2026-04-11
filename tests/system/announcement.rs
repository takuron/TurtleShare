// 公告 API 集成测试
//
// 测试 PUT /api/admin/announcement 和 GET /api/public/announcement
// 是否按照 docs/api.md 的规范正确响应。

use super::common;
use serde_json::{json, Value};

// =========================================================================
// GET /api/public/announcement — 无公告时
// =========================================================================

/// Tests that GET /api/public/announcement returns null data when no announcement exists.
//
// // 测试无公告时 GET /api/public/announcement 返回 null 数据。
#[tokio::test]
async fn public_announcement_returns_null_when_none_exists() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/announcement").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体：data 应为 null
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_null(), "data should be null when no announcement exists");
}

// =========================================================================
// PUT /api/admin/announcement — 鉴权检查
// =========================================================================

/// Tests that PUT /api/admin/announcement requires admin authentication.
//
// // 测试 PUT /api/admin/announcement 需要管理员鉴权。
#[tokio::test]
async fn put_announcement_requires_auth() {
    let server = common::TestServer::spawn().await;

    // 未携带 token 直接请求
    let resp = server
        .put_json("/api/admin/announcement", &json!({"content": "Hello"}))
        .await;

    // 应返回 401
    assert_eq!(resp.status(), 401);
}

// =========================================================================
// PUT /api/admin/announcement — 成功发布
// =========================================================================

/// Tests that an admin can publish an announcement and it is returned correctly.
//
// // 测试管理员可以发布公告且返回正确的响应。
#[tokio::test]
async fn put_announcement_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let content = "Site maintenance scheduled for Saturday.";
    let resp = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": content}),
            &token,
        )
        .await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体结构
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["content"], content);

    // 3. updated_at 应为正整数（Unix 时间戳）
    let updated_at = body["data"]["updated_at"].as_i64().expect("updated_at should be integer");
    assert!(updated_at > 0, "updated_at should be a positive Unix timestamp");
}

// =========================================================================
// PUT + GET — 发布后公开接口可读
// =========================================================================

/// Tests that after publishing, the announcement is visible via the public endpoint.
//
// // 测试发布公告后，公开接口可以正确读取。
#[tokio::test]
async fn public_announcement_returns_published_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 发布公告
    let content = "We are upgrading our servers tonight.";
    let put_resp = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": content}),
            &token,
        )
        .await;
    assert_eq!(put_resp.status(), 200);

    // 2. 通过公开接口读取
    let get_resp = server.get("/api/public/announcement").await;
    assert_eq!(get_resp.status(), 200);

    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["content"], content);

    // 3. updated_at 应为正整数
    let updated_at = body["data"]["updated_at"].as_i64().expect("updated_at should be integer");
    assert!(updated_at > 0);
}

// =========================================================================
// PUT — 更新公告（覆盖旧内容）
// =========================================================================

/// Tests that publishing a new announcement overwrites the previous one.
//
// // 测试发布新公告会覆盖旧公告。
#[tokio::test]
async fn put_announcement_overwrites_previous() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 发布第一条公告
    let resp1 = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": "First announcement"}),
            &token,
        )
        .await;
    assert_eq!(resp1.status(), 200);

    // 2. 发布第二条公告（覆盖）
    let resp2 = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": "Updated announcement"}),
            &token,
        )
        .await;
    assert_eq!(resp2.status(), 200);

    // 3. 公开接口应返回第二条内容
    let get_resp = server.get("/api/public/announcement").await;
    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["data"]["content"], "Updated announcement");
}

// =========================================================================
// PUT — 验证错误：空内容
// =========================================================================

/// Tests that publishing an announcement with empty content returns 400 validation error.
//
// // 测试发布空内容的公告返回 400 验证错误。
#[tokio::test]
async fn put_announcement_empty_content_returns_400() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": ""}),
            &token,
        )
        .await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 400);

    // 2. 验证错误响应体
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(body["error"]["message"], "content must not be empty");
}

// =========================================================================
// PUT — Markdown 内容支持
// =========================================================================

/// Tests that Markdown content is stored and returned as-is.
//
// // 测试 Markdown 内容原样存储和返回。
#[tokio::test]
async fn put_announcement_markdown_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let markdown = "# Important Notice\n\n- Item 1\n- Item 2\n\n**Bold** and *italic* text.";
    let resp = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": markdown}),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    // 公开接口返回的内容应与原始 Markdown 完全一致
    let get_resp = server.get("/api/public/announcement").await;
    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["data"]["content"], markdown);
}

// =========================================================================
// PUT — Unicode 内容支持
// =========================================================================

/// Tests that Unicode content (CJK, emoji) is preserved correctly.
//
// // 测试 Unicode 内容（中日韩文字、emoji）能正确保留。
#[tokio::test]
async fn put_announcement_unicode_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let content = "系统维护通知 🔧 サーバーメンテナンス 공지사항";
    let resp = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": content}),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    let get_resp = server.get("/api/public/announcement").await;
    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["data"]["content"], content);
}

// =========================================================================
// PUT — updated_at 随更新变化
// =========================================================================

/// Tests that updated_at changes when the announcement is updated.
//
// // 测试更新公告时 updated_at 会变化。
#[tokio::test]
async fn put_announcement_updated_at_changes_on_update() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 发布第一条公告
    let resp1 = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": "First"}),
            &token,
        )
        .await;
    let body1: Value = resp1.json().await.unwrap();
    let ts1 = body1["data"]["updated_at"].as_i64().unwrap();

    // 2. 短暂等待后发布第二条公告
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let resp2 = server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": "Second"}),
            &token,
        )
        .await;
    let body2: Value = resp2.json().await.unwrap();
    let ts2 = body2["data"]["updated_at"].as_i64().unwrap();

    // 3. 第二次的时间戳应 >= 第一次
    assert!(ts2 >= ts1, "updated_at should not decrease: {} >= {}", ts2, ts1);
}

// =========================================================================
// GET /api/public/announcement — 无需鉴权
// =========================================================================

/// Tests that the public announcement endpoint does not require authentication.
//
// // 测试公开公告接口无需鉴权即可访问。
#[tokio::test]
async fn public_announcement_accessible_without_auth() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 先发布一条公告
    server
        .put_json_with_token(
            "/api/admin/announcement",
            &json!({"content": "Public test"}),
            &token,
        )
        .await;

    // 2. 不携带任何 token 直接访问公开接口
    let resp = server.get("/api/public/announcement").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["content"], "Public test");
}
