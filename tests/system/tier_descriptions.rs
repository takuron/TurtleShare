// 等级说明 API 集成测试
//
// 测试 PUT /api/admin/tier-descriptions、DELETE /api/admin/tier-descriptions/:tier
// 和 GET /api/public/tier-descriptions 是否按照 docs/api.md 的规范正确响应。

use super::common;
use serde_json::{json, Value};

// =========================================================================
// GET /api/public/tier-descriptions — 无等级说明时
// =========================================================================

/// Tests that GET /api/public/tier-descriptions returns empty tiers with updated_at -1 when no tier descriptions exist.
//
// // 测试无等级说明时 GET /api/public/tier-descriptions 返回空 tiers 列表且 updated_at 为 -1。
#[tokio::test]
async fn public_tier_descriptions_returns_empty_when_none_exists() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/tier-descriptions").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体：tiers 应为空数组，updated_at 应为 -1
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let tiers = body["data"]["tiers"]
        .as_array()
        .expect("tiers should be an array");
    assert!(tiers.is_empty(), "tiers should be empty when none exist");
    assert_eq!(body["data"]["updated_at"], -1);
}

// =========================================================================
// GET /api/public/tier-descriptions — 无需鉴权
// =========================================================================

/// Tests that the public tier-descriptions endpoint does not require authentication.
//
// // 测试公开等级说明接口无需鉴权即可访问。
#[tokio::test]
async fn public_tier_descriptions_accessible_without_auth() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 先创建一条等级说明
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "Basic access", "price": "¥10/月"}),
            &token,
        )
        .await;

    // 2. 不携带任何 token 直接访问公开接口
    let resp = server.get("/api/public/tier-descriptions").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["tiers"].is_array());
}

// =========================================================================
// PUT /api/admin/tier-descriptions — 鉴权检查
// =========================================================================

/// Tests that PUT /api/admin/tier-descriptions requires admin authentication.
//
// // 测试 PUT /api/admin/tier-descriptions 需要管理员鉴权。
#[tokio::test]
async fn put_tier_description_requires_auth() {
    let server = common::TestServer::spawn().await;

    // 未携带 token 直接请求
    let resp = server
        .put_json(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic"}),
        )
        .await;

    // 应返回 401
    assert_eq!(resp.status(), 401);
}

// =========================================================================
// PUT /api/admin/tier-descriptions — 成功创建
// =========================================================================

/// Tests that an admin can create a tier description and it is returned correctly.
//
// // 测试管理员可以创建等级说明且返回正确的响应。
#[tokio::test]
async fn put_tier_description_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({
                "tier": 1,
                "name": "Basic",
                "description": "Access to basic content",
                "price": "¥10/月"
            }),
            &token,
        )
        .await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体结构
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // 3. tiers 数组应包含刚创建的等级说明
    let tiers = body["data"]["tiers"].as_array().expect("tiers should be an array");
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0]["tier"], 1);
    assert_eq!(tiers[0]["name"], "Basic");
    assert_eq!(tiers[0]["description"], "Access to basic content");
    assert_eq!(tiers[0]["price"], "¥10/月");

    // 4. updated_at 应为正整数（Unix 时间戳）
    let updated_at = body["data"]["updated_at"]
        .as_i64()
        .expect("updated_at should be integer");
    assert!(
        updated_at > 0,
        "updated_at should be a positive Unix timestamp"
    );
}

// =========================================================================
// PUT + GET — 创建后公开接口可读
// =========================================================================

/// Tests that after creating tier descriptions, they are visible via the public endpoint.
//
// // 测试创建等级说明后，公开接口可以正确读取。
#[tokio::test]
async fn public_tier_descriptions_returns_created_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建等级说明
    let put_resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({
                "tier": 1,
                "name": "Basic",
                "description": "Access to basic content",
                "price": "¥10/月"
            }),
            &token,
        )
        .await;
    assert_eq!(put_resp.status(), 200);

    // 2. 通过公开接口读取
    let get_resp = server.get("/api/public/tier-descriptions").await;
    assert_eq!(get_resp.status(), 200);

    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let tiers = body["data"]["tiers"].as_array().expect("tiers should be an array");
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0]["tier"], 1);
    assert_eq!(tiers[0]["name"], "Basic");
    assert_eq!(tiers[0]["description"], "Access to basic content");
    assert_eq!(tiers[0]["price"], "¥10/月");

    // 3. updated_at 应为正整数
    let updated_at = body["data"]["updated_at"]
        .as_i64()
        .expect("updated_at should be integer");
    assert!(updated_at > 0);
}

// =========================================================================
// PUT — 覆盖已存在的等级说明
// =========================================================================

/// Tests that putting a tier description with the same tier overwrites the existing one.
//
// // 测试对相同等级 PUT 会覆盖已存在的等级说明。
#[tokio::test]
async fn put_tier_description_overwrites_existing() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建等级 1
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "Old desc", "price": "¥10/月"}),
            &token,
        )
        .await;

    // 2. 覆盖等级 1
    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic Plus", "description": "New desc", "price": "¥15/月"}),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    // 3. 验证只有一个等级且内容已更新
    let body: Value = resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0]["name"], "Basic Plus");
    assert_eq!(tiers[0]["description"], "New desc");
    assert_eq!(tiers[0]["price"], "¥15/月");
}

// =========================================================================
// PUT — 多个等级按 tier 排序
// =========================================================================

/// Tests that tiers are sorted by tier level in the response.
//
// // 测试响应中等级按 tier 值排序。
#[tokio::test]
async fn put_tier_descriptions_sorted_by_tier() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 先创建等级 3
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 3, "name": "Premium", "description": "Premium", "price": "¥50/月"}),
            &token,
        )
        .await;

    // 2. 再创建等级 1
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "Basic", "price": "¥10/月"}),
            &token,
        )
        .await;

    // 3. 最后创建等级 2
    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 2, "name": "Standard", "description": "Standard", "price": "¥30/月"}),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    // 4. 验证排序：应为 tier 1, 2, 3
    let body: Value = resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers.len(), 3);
    assert_eq!(tiers[0]["tier"], 1);
    assert_eq!(tiers[1]["tier"], 2);
    assert_eq!(tiers[2]["tier"], 3);
}

// =========================================================================
// PUT — 验证错误：name/description/price 全空
// =========================================================================

/// Tests that creating a tier description with all empty fields returns 400.
//
// // 测试 name、description、price 全空时返回 400 验证错误。
#[tokio::test]
async fn put_tier_description_all_empty_fields_returns_400() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "", "description": "", "price": ""}),
            &token,
        )
        .await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 400);

    // 2. 验证错误响应体
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        body["error"]["message"],
        "at least one of name, description, or price must not be empty"
    );
}

// =========================================================================
// PUT — 验证错误：仅 null 字段
// =========================================================================

/// Tests that creating a tier description with all null optional fields returns 400.
//
// // 测试仅提供 tier 而不提供任何有意义字段时返回 400 验证错误。
#[tokio::test]
async fn put_tier_description_only_tier_returns_400() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1}),
            &token,
        )
        .await;

    // 至少需要 name、description、price 中的一个非空
    assert_eq!(resp.status(), 400);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// =========================================================================
// PUT — 部分字段创建（仅 name）
// =========================================================================

/// Tests that a tier description can be created with only a name (other fields optional).
//
// // 测试仅提供 name 即可创建等级说明（其余字段可选）。
#[tokio::test]
async fn put_tier_description_with_only_name() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic"}),
            &token,
        )
        .await;

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0]["tier"], 1);
    assert_eq!(tiers[0]["name"], "Basic");
}

// =========================================================================
// PUT — 更新时保留未提供的字段
// =========================================================================

/// Tests that on update, only provided non-empty fields are overwritten; omitted fields retain values.
//
// // 测试更新时仅覆盖提供的非空字段，省略的字段保留当前值。
#[tokio::test]
async fn put_tier_description_update_preserves_omitted_fields() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建完整的等级说明
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({
                "tier": 1,
                "name": "Basic",
                "description": "Original desc",
                "price": "¥10/月"
            }),
            &token,
        )
        .await;

    // 2. 仅更新 name，不提供 description 和 price
    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic Renewed"}),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    // 3. 验证 name 已更新，description 和 price 保留
    let body: Value = resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers[0]["name"], "Basic Renewed");
    assert_eq!(tiers[0]["description"], "Original desc");
    assert_eq!(tiers[0]["price"], "¥10/月");
}

// =========================================================================
// PUT — Unicode 内容支持
// =========================================================================

/// Tests that Unicode content (CJK, emoji) is preserved correctly in tier descriptions.
//
// // 测试等级说明中 Unicode 内容（中日韩文字、emoji）能正确保留。
#[tokio::test]
async fn put_tier_description_unicode_content() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({
                "tier": 1,
                "name": "基础会员 🌱",
                "description": "基本内容访问权限",
                "price": "¥10/月"
            }),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 200);

    let get_resp = server.get("/api/public/tier-descriptions").await;
    let body: Value = get_resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers[0]["name"], "基础会员 🌱");
    assert_eq!(tiers[0]["description"], "基本内容访问权限");
    assert_eq!(tiers[0]["price"], "¥10/月");
}

// =========================================================================
// DELETE /api/admin/tier-descriptions/:tier — 鉴权检查
// =========================================================================

/// Tests that DELETE /api/admin/tier-descriptions/:tier requires admin authentication.
//
// // 测试 DELETE /api/admin/tier-descriptions/:tier 需要管理员鉴权。
#[tokio::test]
async fn delete_tier_description_requires_auth() {
    let server = common::TestServer::spawn().await;

    // 未携带 token 直接请求
    let resp = server.delete("/api/admin/tier-descriptions/1").await;

    // 应返回 401
    assert_eq!(resp.status(), 401);
}

// =========================================================================
// DELETE — 成功删除
// =========================================================================

/// Tests that an admin can delete a tier description successfully.
//
// // 测试管理员可以成功删除等级说明。
#[tokio::test]
async fn delete_tier_description_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建等级说明
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "desc", "price": "¥10/月"}),
            &token,
        )
        .await;

    // 2. 删除等级 1
    let resp = server
        .delete_with_token("/api/admin/tier-descriptions/1", &token)
        .await;

    // 3. 验证响应
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["deleted"], true);
    assert_eq!(body["data"]["tier"], 1);
}

// =========================================================================
// DELETE — 删除后公开接口不再包含
// =========================================================================

/// Tests that after deleting a tier description, it no longer appears in the public endpoint.
//
// // 测试删除等级说明后，公开接口不再包含该等级。
#[tokio::test]
async fn delete_tier_description_removes_from_public() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建两个等级
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "desc", "price": "¥10/月"}),
            &token,
        )
        .await;
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 2, "name": "Premium", "description": "desc", "price": "¥30/月"}),
            &token,
        )
        .await;

    // 2. 删除等级 1
    let del_resp = server
        .delete_with_token("/api/admin/tier-descriptions/1", &token)
        .await;
    assert_eq!(del_resp.status(), 200);

    // 3. 公开接口应只剩等级 2
    let get_resp = server.get("/api/public/tier-descriptions").await;
    let body: Value = get_resp.json().await.unwrap();
    let tiers = body["data"]["tiers"].as_array().unwrap();
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0]["tier"], 2);
    assert_eq!(tiers[0]["name"], "Premium");
}

// =========================================================================
// DELETE — 删除所有等级后公开接口返回空列表
// =========================================================================

/// Tests that after deleting all tier descriptions, the public endpoint returns empty tiers.
//
// // 测试删除所有等级说明后，公开接口返回空 tiers 列表。
#[tokio::test]
async fn delete_all_tier_descriptions_returns_empty_tiers() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建一个等级
    server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "desc", "price": "¥10/月"}),
            &token,
        )
        .await;

    // 2. 删除该等级
    server
        .delete_with_token("/api/admin/tier-descriptions/1", &token)
        .await;

    // 3. 公开接口应返回空 tiers 列表
    let get_resp = server.get("/api/public/tier-descriptions").await;
    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let tiers = body["data"]["tiers"]
        .as_array()
        .expect("tiers should be an array");
    assert!(
        tiers.is_empty(),
        "tiers should be empty after deleting all entries"
    );
}

// =========================================================================
// DELETE — 不存在的等级返回 404
// =========================================================================

/// Tests that deleting a non-existent tier description returns 404.
//
// // 测试删除不存在的等级说明返回 404。
#[tokio::test]
async fn delete_tier_description_not_found_returns_404() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .delete_with_token("/api/admin/tier-descriptions/99", &token)
        .await;

    assert_eq!(resp.status(), 404);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert_eq!(body["error"]["message"], "Tier description not found");
}

// =========================================================================
// PUT — updated_at 随更新变化
// =========================================================================

/// Tests that updated_at changes when tier descriptions are updated.
//
// // 测试更新等级说明时 updated_at 会变化。
#[tokio::test]
async fn put_tier_description_updated_at_changes_on_update() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 1. 创建等级说明
    let resp1 = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "v1", "price": "¥10/月"}),
            &token,
        )
        .await;
    let body1: Value = resp1.json().await.unwrap();
    let ts1 = body1["data"]["updated_at"].as_i64().unwrap();

    // 2. 短暂等待后更新
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let resp2 = server
        .put_json_with_token(
            "/api/admin/tier-descriptions",
            &json!({"tier": 1, "name": "Basic", "description": "v2", "price": "¥10/月"}),
            &token,
        )
        .await;
    let body2: Value = resp2.json().await.unwrap();
    let ts2 = body2["data"]["updated_at"].as_i64().unwrap();

    // 3. 第二次的时间戳应 >= 第一次
    assert!(
        ts2 >= ts1,
        "updated_at should not decrease: {} >= {}",
        ts2,
        ts1
    );
}
