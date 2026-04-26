// 健康检查与站点信息 API 集成测试
//
// 测试 /api/health、/api 以及 /api/public/site-info 端点
// 是否按照 docs/api.md 的规范正确响应。

use super::common::{self, TestConfig};
use serde_json::Value;

// =========================================================================
// 健康检查
// =========================================================================

/// Tests that GET /api/health returns 200 with {"success": true, "data": {"status": "ok"}}.
//
// // 测试 GET /api/health 返回 200 和 {"success": true, "data": {"status": "ok"}}。
#[tokio::test]
async fn health_check_returns_ok() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/health").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应体结构
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "ok");
}

/// Tests that GET /api returns the plain text status message.
//
// // 测试 GET /api 返回纯文本状态消息。
#[tokio::test]
async fn api_root_returns_running_message() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证响应内容
    let text = resp.text().await.unwrap();
    assert_eq!(text, "TurtleShare API is running!");
}

/// Tests that requesting a non-existent API path returns 404.
//
// // 测试请求不存在的 API 路径返回 404。
#[tokio::test]
async fn nonexistent_api_path_returns_404() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/nonexistent").await;

    // 注意：由于 SPA fallback，非 /api 路径可能返回 200
    // 但 /api 前缀下不存在的路径应该返回 404
    assert_eq!(resp.status(), 404);
}

// =========================================================================
// site-info：基本字段映射
// =========================================================================

/// Tests that the default config's basic fields are returned correctly.
//
// // 测试默认配置的基本字段能正确返回。
#[tokio::test]
async fn site_info_returns_default_values() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/public/site-info").await;

    // 1. 验证状态码
    assert_eq!(resp.status(), 200);

    // 2. 验证默认配置值
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "TurtleShare-Test");
    assert_eq!(body["data"]["author"], "TestAdmin");
}

/// Tests that custom string, boolean, and integer scalars are forwarded.
//
// // 测试自定义的字符串、布尔值、整数标量能正确透传。
#[tokio::test]
async fn site_info_custom_scalars() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r##"[siteinfo]
name = "CustomSite"
theme_color = "#ff5500"
show_sidebar = true
max_items_per_page = 25
rating = 4.5
"##
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 1. 字符串标量
    assert_eq!(data["name"], "CustomSite");
    assert_eq!(data["theme_color"], "#ff5500");

    // 2. 布尔值标量
    assert_eq!(data["show_sidebar"], true);

    // 3. 整数标量
    assert_eq!(data["max_items_per_page"], 25);

    // 4. 浮点数标量
    assert_eq!(data["rating"], 4.5);
}

// =========================================================================
// site-info：TOML 数组映射
// =========================================================================

/// Tests that TOML arrays are correctly mapped to JSON arrays.
//
// // 测试 TOML 数组能正确映射为 JSON 数组。
#[tokio::test]
async fn site_info_array_values() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "ArrayTest"
nav_links = ["Home", "About", "Contact"]
featured_ids = [1, 2, 3]
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 1. 字符串数组
    let nav_links = data["nav_links"]
        .as_array()
        .expect("nav_links should be array");
    assert_eq!(nav_links.len(), 3);
    assert_eq!(nav_links[0], "Home");
    assert_eq!(nav_links[1], "About");
    assert_eq!(nav_links[2], "Contact");

    // 2. 整数数组
    let featured_ids = data["featured_ids"]
        .as_array()
        .expect("featured_ids should be array");
    assert_eq!(featured_ids.len(), 3);
    assert_eq!(featured_ids[0], 1);
    assert_eq!(featured_ids[2], 3);
}

// =========================================================================
// site-info：TOML 子表 / 嵌套结构映射
// =========================================================================

/// Tests that TOML sub-tables (nested objects) are mapped to JSON objects.
//
// // 测试 TOML 子表（嵌套对象）能正确映射为 JSON 对象。
#[tokio::test]
async fn site_info_nested_table() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r##"[siteinfo]
name = "NestedTest"

[siteinfo.social]
twitter = "https://twitter.com/example"
github = "https://github.com/example"

[siteinfo.theme]
primary_color = "#3498db"
dark_mode = false
font_size = 16
"##
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 1. 嵌套对象应为 JSON object
    assert!(data["social"].is_object(), "social should be an object");
    assert_eq!(data["social"]["twitter"], "https://twitter.com/example");
    assert_eq!(data["social"]["github"], "https://github.com/example");

    // 2. 另一个嵌套对象，含混合类型
    assert!(data["theme"].is_object(), "theme should be an object");
    assert_eq!(data["theme"]["primary_color"], "#3498db");
    assert_eq!(data["theme"]["dark_mode"], false);
    assert_eq!(data["theme"]["font_size"], 16);
}

/// Tests that inline tables in TOML are correctly mapped.
//
// // 测试 TOML 内联表能正确映射。
#[tokio::test]
async fn site_info_inline_table() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "InlineTest"
logo = { url = "/img/logo.png", alt = "Site Logo", width = 200 }
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert!(data["logo"].is_object());
    assert_eq!(data["logo"]["url"], "/img/logo.png");
    assert_eq!(data["logo"]["alt"], "Site Logo");
    assert_eq!(data["logo"]["width"], 200);
}

/// Tests array-of-tables (deeply nested TOML structure).
//
// // 测试 TOML 表数组（深层嵌套结构）。
#[tokio::test]
async fn site_info_array_of_tables() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "AOTTest"

[[siteinfo.menu_items]]
label = "Home"
href = "/"
icon = "home"

[[siteinfo.menu_items]]
label = "Blog"
href = "/blog"
icon = "book"

[[siteinfo.menu_items]]
label = "About"
href = "/about"
icon = "info"
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 表数组应映射为 JSON 对象数组
    let items = data["menu_items"]
        .as_array()
        .expect("menu_items should be array");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["label"], "Home");
    assert_eq!(items[0]["href"], "/");
    assert_eq!(items[1]["label"], "Blog");
    assert_eq!(items[2]["icon"], "info");
}

// =========================================================================
// site-info：空配置与边界情况
// =========================================================================

/// Tests that an empty [siteinfo] section returns an empty JSON object.
//
// // 测试空的 [siteinfo] 部分返回空 JSON 对象。
#[tokio::test]
async fn site_info_empty_section() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // 空 section 应该返回空对象 {}
    let data = body["data"].as_object().expect("data should be an object");
    assert!(
        data.is_empty(),
        "empty [siteinfo] should produce empty object"
    );
}

/// Tests that empty string values are preserved (not null).
//
// // 测试空字符串值被保留（而非 null）。
#[tokio::test]
async fn site_info_empty_string_values() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = ""
description = ""
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 空字符串应保留为 ""，而非 null
    assert_eq!(data["name"], "");
    assert_eq!(data["description"], "");
}

// =========================================================================
// site-info：特殊字符与安全性
// =========================================================================

/// Tests that HTML/script content in values is treated as plain strings (no XSS risk).
//
// // 测试值中的 HTML/脚本内容被视为纯字符串（无 XSS 风险）。
#[tokio::test]
async fn site_info_html_content_passthrough() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "<script>alert('xss')</script>"
description = "<img src=x onerror=alert(1)>"
footer = "normal & safe < text > here"
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 内容应作为纯字符串原样返回，由前端负责转义
    assert_eq!(data["name"], "<script>alert('xss')</script>");
    assert_eq!(data["description"], "<img src=x onerror=alert(1)>");
    assert_eq!(data["footer"], "normal & safe < text > here");
}

/// Tests that Unicode content including CJK, emoji, and RTL text is preserved.
//
// // 测试 Unicode 内容（包括中日韩文字、emoji、RTL 文字）能正确保留。
#[tokio::test]
async fn site_info_unicode_content() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "龟速分享站 🐢"
greeting = "مرحبا بكم"
motto = "亀の歩みで確実に 🎯"
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["name"], "龟速分享站 🐢");
    assert_eq!(data["greeting"], "مرحبا بكم");
    assert_eq!(data["motto"], "亀の歩みで確実に 🎯");
}

/// Tests that very long string values are handled without truncation.
//
// // 测试超长字符串值不会被截断。
#[tokio::test]
async fn site_info_long_string_value() {
    let long_value = "A".repeat(10_000);
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(format!(
        "[siteinfo]\nname = \"LongTest\"\nlong_field = \"{}\"\n",
        long_value
    ));

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let returned = body["data"]["long_field"].as_str().unwrap();
    assert_eq!(returned.len(), 10_000);
    assert_eq!(returned, long_value);
}

/// Tests that TOML special characters in string values are correctly handled.
//
// // 测试 TOML 特殊字符在字符串值中能正确处理。
#[tokio::test]
async fn site_info_toml_special_chars() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r#"[siteinfo]
name = "Test \"Quoted\" Site"
path = "C:\\Users\\test"
multiline = "line1\nline2\ttabbed"
"#
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["name"], "Test \"Quoted\" Site");
    assert_eq!(data["path"], "C:\\Users\\test");
    assert_eq!(data["multiline"], "line1\nline2\ttabbed");
}

// =========================================================================
// site-info：确保不泄露其他配置节
// =========================================================================

/// Tests that only [siteinfo] data is exposed — no admin, jwt, database, or server secrets.
//
// // 测试仅暴露 [siteinfo] 数据——不泄露管理员、JWT、数据库或服务器密钥。
#[tokio::test]
async fn site_info_does_not_leak_other_config_sections() {
    let server = common::TestServer::spawn().await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 不应包含任何其他配置节的字段
    assert!(data.get("admin").is_none(), "admin config must not leak");
    assert!(
        data.get("password_hash").is_none(),
        "password_hash must not leak"
    );
    assert!(data.get("jwt").is_none(), "jwt config must not leak");
    assert!(
        data.get("base_secret").is_none(),
        "base_secret must not leak"
    );
    assert!(
        data.get("database").is_none(),
        "database config must not leak"
    );
    assert!(data.get("server").is_none(), "server config must not leak");
    assert!(
        data.get("storage").is_none(),
        "storage config must not leak"
    );
    assert!(data.get("host").is_none(), "host must not leak");
    assert!(data.get("port").is_none(), "port must not leak");

    // data 应该只包含 [siteinfo] 中定义的键
    let obj = data.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            ["name", "author", "sponsor_link", "header_image"].contains(&key.as_str()),
            "unexpected key '{}' in site-info response",
            key
        );
    }
}

// =========================================================================
// site-info：复杂混合结构
// =========================================================================

/// Tests a realistic, complex configuration with mixed types at multiple levels.
//
// // 测试一个包含多层次混合类型的真实复杂配置。
#[tokio::test]
async fn site_info_complex_mixed_config() {
    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(
        r##"[siteinfo]
name = "TurtleShare Pro"
version = "2.0"
maintenance_mode = false
max_display_items = 50

[siteinfo.branding]
primary_color = "#2ecc71"
secondary_color = "#3498db"
logo_url = "/assets/logo.svg"

[siteinfo.features]
dark_mode = true
i18n = true
supported_langs = ["zh-CN", "en-US", "ja-JP"]

[[siteinfo.announcements]]
title = "Welcome!"
message = "Site is live."
priority = 1

[[siteinfo.announcements]]
title = "Update"
message = "New features added."
priority = 2
"##
        .to_string(),
    );

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    // 1. 顶级标量
    assert_eq!(data["name"], "TurtleShare Pro");
    assert_eq!(data["maintenance_mode"], false);
    assert_eq!(data["max_display_items"], 50);

    // 2. 嵌套对象
    assert_eq!(data["branding"]["primary_color"], "#2ecc71");
    assert_eq!(data["branding"]["logo_url"], "/assets/logo.svg");

    // 3. 嵌套对象中的数组
    let langs = data["features"]["supported_langs"]
        .as_array()
        .expect("supported_langs should be array");
    assert_eq!(langs.len(), 3);
    assert_eq!(langs[0], "zh-CN");

    // 4. 表数组
    let announcements = data["announcements"]
        .as_array()
        .expect("announcements should be array");
    assert_eq!(announcements.len(), 2);
    assert_eq!(announcements[0]["title"], "Welcome!");
    assert_eq!(announcements[1]["priority"], 2);
}

/// Tests that many custom keys can coexist without interference.
//
// // 测试大量自定义键能共存而不互相干扰。
#[tokio::test]
async fn site_info_many_keys() {
    let mut entries = vec!["[siteinfo]".to_string()];
    for i in 0..100 {
        entries.push(format!("key_{} = \"value_{}\"", i, i));
    }
    let toml_content = entries.join("\n") + "\n";

    let mut config = TestConfig::default();
    config.siteinfo_toml = Some(toml_content);

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server.get("/api/public/site-info").await;
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_object().expect("data should be object");

    // 验证所有 100 个键都存在且值正确
    assert_eq!(data.len(), 100);
    for i in 0..100 {
        let key = format!("key_{}", i);
        let expected = format!("value_{}", i);
        assert_eq!(
            data[&key].as_str().unwrap(),
            expected,
            "key '{}' mismatch",
            key
        );
    }
}
