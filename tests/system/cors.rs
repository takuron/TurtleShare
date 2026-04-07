use super::common::{self, TestConfig};
use reqwest::Method;
use serde_json::Value;

#[tokio::test]
async fn same_origin_requests_are_allowed_by_default() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .get(server.url("/api/health"))
        .header("Origin", &server.base_url)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some(server.base_url.as_str())
    );
}

#[tokio::test]
async fn whitelisted_cross_origin_requests_are_allowed() {
    let origin = "https://admin.example.com";
    let mut config = TestConfig::default();
    config.cors_origins = vec![origin.to_string()];

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server
        .client
        .get(server.url("/api/public/site-info"))
        .header("Origin", origin)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some(origin)
    );
}

#[tokio::test]
async fn non_whitelisted_origins_are_rejected() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .get(server.url("/api/public/site-info"))
        .header("Origin", "https://evil.example.com")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert_eq!(body["error"]["message"], "CORS origin is not allowed");
}

#[tokio::test]
async fn allowed_preflight_requests_bypass_authentication() {
    let origin = "https://admin.example.com";
    let mut config = TestConfig::default();
    config.cors_origins = vec![origin.to_string()];

    let server = common::TestServer::spawn_with_config(config).await;
    let resp = server
        .client
        .request(Method::OPTIONS, server.url("/api/users/password"))
        .header("Origin", origin)
        .header("Access-Control-Request-Method", "PUT")
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type",
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204);
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some(origin)
    );
    assert_eq!(
        resp.headers()
            .get("access-control-allow-methods")
            .and_then(|value| value.to_str().ok()),
        Some("PUT")
    );
    assert_eq!(
        resp.headers()
            .get("access-control-allow-headers")
            .and_then(|value| value.to_str().ok()),
        Some("authorization,content-type")
    );
}

#[tokio::test]
async fn non_whitelisted_preflight_requests_are_rejected() {
    let server = common::TestServer::spawn().await;

    let resp = server
        .client
        .request(Method::OPTIONS, server.url("/api/users/password"))
        .header("Origin", "https://evil.example.com")
        .header("Access-Control-Request-Method", "PUT")
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type",
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert_eq!(body["error"]["message"], "CORS origin is not allowed");
}
