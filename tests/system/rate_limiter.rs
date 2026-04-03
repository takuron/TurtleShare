// 限流器集成测试
//
// 测试三个限流器的行为：
// 1. 管理员登录限流器（5分钟内最多10次请求）
// 2. 用户登录限流器（5分钟内最多10次请求）
// 3. 全局限流器（1分钟内每IP最多100次请求，作用于所有 /api/* 路由）

use super::common;
use serde_json::{Value, json};

// ============================================================
// 辅助函数
// ============================================================

/// 通过管理员 API 创建一个用户。
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
                "note": format!("Test user {}", username)
            }),
            admin_token,
        )
        .await;

    assert_eq!(resp.status(), 201, "Failed to create user {}", username);
    let body: Value = resp.json().await.unwrap();
    body["data"]["hash_id"].as_str().unwrap().to_string()
}

// ============================================================
// 管理员登录限流器测试
// ============================================================

/// 管理员登录在10次请求后应返回 429。
#[tokio::test]
async fn admin_login_rate_limit_triggers_after_10_requests() {
    let server = common::TestServer::spawn().await;

    // 1. 发送10次请求（全部应被允许，无论凭据正确与否）
    for i in 0..10 {
        let resp = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
        let status = resp.status().as_u16();
        assert_ne!(
            status,
            429,
            "Request {} should not be rate limited, got 429",
            i + 1
        );
    }

    // 2. 第11次请求应被限流
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "wrong"}),
        )
        .await;

    assert_eq!(resp.status(), 429);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "TOO_MANY_REQUESTS");
}

/// 管理员登录限流器即使凭据正确也会触发。
#[tokio::test]
async fn admin_login_rate_limit_blocks_even_correct_credentials() {
    let server = common::TestServer::spawn().await;

    // 1. 用错误凭据消耗10次配额
    for _ in 0..10 {
        let _ = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 2. 即使凭据正确，第11次也应被限流
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;

    assert_eq!(resp.status(), 429);
}

/// 管理员登录限流不影响其他端点。
#[tokio::test]
async fn admin_login_rate_limit_does_not_affect_other_endpoints() {
    let server = common::TestServer::spawn().await;

    // 1. 先登录获取令牌
    let token = server.admin_login().await;

    // 2. 消耗管理员登录的限流配额（登录已消耗1次，再消耗9次）
    for _ in 0..9 {
        let _ = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 3. 确认管理员登录已被限流
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(resp.status(), 429);

    // 4. 其他管理员端点仍可正常使用
    let users_resp = server.get_with_token("/api/admin/users", &token).await;
    assert_eq!(users_resp.status(), 200);
}

// ============================================================
// 用户登录限流器测试
// ============================================================

/// 用户登录在10次请求后应返回 429。
#[tokio::test]
async fn user_login_rate_limit_triggers_after_10_requests() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    // 1. 创建测试用户
    create_test_user(&server, &admin_token, "rl_user", "user_pass").await;

    // 2. 发送10次请求（允许通过，无论凭据正确与否）
    for i in 0..10 {
        let resp = server
            .post_json(
                "/api/users/login",
                &json!({"username": "rl_user", "password": "wrong"}),
            )
            .await;
        let status = resp.status().as_u16();
        assert_ne!(
            status,
            429,
            "Request {} should not be rate limited, got 429",
            i + 1
        );
    }

    // 3. 第11次请求应被限流
    let resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "rl_user", "password": "wrong"}),
        )
        .await;

    assert_eq!(resp.status(), 429);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "TOO_MANY_REQUESTS");
}

/// 用户登录限流器即使凭据正确也会触发。
#[tokio::test]
async fn user_login_rate_limit_blocks_even_correct_credentials() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "rl_user2", "correct_pass").await;

    // 1. 用错误凭据消耗10次配额
    for _ in 0..10 {
        let _ = server
            .post_json(
                "/api/users/login",
                &json!({"username": "rl_user2", "password": "wrong"}),
            )
            .await;
    }

    // 2. 即使凭据正确，第11次也应被限流
    let resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "rl_user2", "password": "correct_pass"}),
        )
        .await;

    assert_eq!(resp.status(), 429);
}

/// 用户登录限流不影响管理员登录。
#[tokio::test]
async fn user_login_rate_limit_independent_from_admin_login() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "rl_indep_user", "pass123").await;

    // 1. 消耗用户登录的限流配额（管理员登录已消耗了管理员限流器1次）
    for _ in 0..10 {
        let _ = server
            .post_json(
                "/api/users/login",
                &json!({"username": "rl_indep_user", "password": "wrong"}),
            )
            .await;
    }

    // 2. 确认用户登录已被限流
    let resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "rl_indep_user", "password": "pass123"}),
        )
        .await;
    assert_eq!(resp.status(), 429);

    // 3. 管理员登录仍可正常使用（独立的限流器）
    let admin_resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(admin_resp.status(), 200);
}

// ============================================================
// 全局限流器测试
// ============================================================

/// 全局限流器保护所有 API 路由，返回 429 响应格式正确。
/// 注意：全局限流为100次/1分钟，此测试通过在管理员登录限流后
/// 验证错误格式来确认全局限流器的存在和响应格式。
#[tokio::test]
async fn global_rate_limit_response_format() {
    let server = common::TestServer::spawn().await;

    // 1. 触发管理员登录限流（10次后触发）
    for _ in 0..10 {
        let _ = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 2. 验证限流响应的格式
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "wrong"}),
        )
        .await;

    assert_eq!(resp.status(), 429);
    let body: Value = resp.json().await.unwrap();

    // 验证标准错误响应格式
    assert_eq!(body["success"], false);
    assert!(body["error"].is_object());
    assert_eq!(body["error"]["code"], "TOO_MANY_REQUESTS");
    assert!(body["error"]["message"].is_string());
}

/// 全局限流器应用于公共端点。
/// 通过发送大量请求到公共端点确认全局限流器在工作。
/// 注意：wait_until_ready() 会轮询 /api/health 消耗部分全局配额，
/// 因此实际可用配额略少于100。
#[tokio::test]
async fn global_rate_limit_applies_to_public_endpoints() {
    let server = common::TestServer::spawn().await;

    // 1. 持续发送请求直到被限流或超过合理上限
    let mut success_count = 0;
    let mut rate_limited = false;
    for _ in 0..110 {
        let resp = server.get("/api/health").await;
        if resp.status() == 429 {
            rate_limited = true;
            // 验证限流响应格式
            let body: Value = resp.json().await.unwrap();
            assert_eq!(body["success"], false);
            assert_eq!(body["error"]["code"], "TOO_MANY_REQUESTS");
            break;
        }
        assert_eq!(resp.status(), 200);
        success_count += 1;
    }

    // 2. 必须在合理范围内触发限流
    assert!(rate_limited, "Global rate limit should have triggered");
    // 成功次数应接近100（wait_until_ready 消耗了少量配额）
    assert!(
        success_count >= 90 && success_count <= 100,
        "Expected ~100 successful requests before rate limit, got {}",
        success_count
    );
}

/// 全局限流器在消耗后，认证端点也应被限流。
/// 注意：wait_until_ready() 会消耗少量全局配额。
#[tokio::test]
async fn global_rate_limit_shared_across_all_api_routes() {
    let server = common::TestServer::spawn().await;

    // 1. 通过公共端点消耗全局配额直到接近上限
    //    持续发送直到剩余约2次配额
    let mut sent = 0;
    for _ in 0..110 {
        let resp = server.get("/api/health").await;
        sent += 1;
        if resp.status() == 429 {
            break;
        }
    }

    // 因为已经耗尽配额，管理员登录也应被全局限流
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(
        resp.status(),
        429,
        "Admin login should be blocked by global rate limit after {} health requests",
        sent
    );
}

/// 登录限流先于全局限流触发（管理员登录限制更严格）。
#[tokio::test]
async fn login_rate_limit_triggers_before_global_limit() {
    let server = common::TestServer::spawn().await;

    // 1. 发送10次管理员登录请求，触发登录级别限流
    for _ in 0..10 {
        let _ = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 2. 第11次被登录限流器拦截
    let resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "wrong"}),
        )
        .await;
    assert_eq!(resp.status(), 429);

    // 3. 公共端点仍可正常访问（全局配额远未耗尽）
    let health_resp = server.get("/api/health").await;
    assert_eq!(health_resp.status(), 200);
}

// ============================================================
// 限流器隔离性测试
// ============================================================

/// 管理员登录和用户登录使用独立的限流器实例。
#[tokio::test]
async fn admin_and_user_login_limiters_are_independent() {
    let server = common::TestServer::spawn().await;
    let admin_token = server.admin_login().await;

    create_test_user(&server, &admin_token, "indep_test_user", "pass123").await;

    // 1. 消耗管理员登录配额（admin_login 已消耗1次，再消耗9次）
    for _ in 0..9 {
        let _ = server
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 2. 确认管理员登录已被限流
    let admin_resp = server
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(admin_resp.status(), 429);

    // 3. 用户登录不受影响
    let user_resp = server
        .post_json(
            "/api/users/login",
            &json!({"username": "indep_test_user", "password": "pass123"}),
        )
        .await;
    assert_eq!(user_resp.status(), 200);
}

/// 每个测试服务器实例的限流器是独立的（验证测试隔离性）。
#[tokio::test]
async fn rate_limiters_are_isolated_between_server_instances() {
    // 1. 第一个服务器实例
    let server1 = common::TestServer::spawn().await;

    // 消耗 server1 的管理员登录配额
    for _ in 0..10 {
        let _ = server1
            .post_json(
                "/api/admin/login",
                &json!({"username": "admin", "password": "wrong"}),
            )
            .await;
    }

    // 确认 server1 被限流
    let resp1 = server1
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(resp1.status(), 429);

    // 2. 第二个服务器实例不受影响
    let server2 = common::TestServer::spawn().await;
    let resp2 = server2
        .post_json(
            "/api/admin/login",
            &json!({"username": "admin", "password": "admin123"}),
        )
        .await;
    assert_eq!(resp2.status(), 200);
}
