// 集成测试公共模块 - 提供黑箱测试基础设施
//
// 核心思路：
// 1. 为每个测试生成独立的临时目录（配置、数据库、存储）
// 2. 启动实际的 TurtleShare 二进制程序，绑定到随机端口
// 3. 等待健康检查通过后返回客户端
// 4. 测试结束后自动清理进程和临时文件

use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tempfile::TempDir;

/// Represents a running TurtleShare server instance for testing.
///
/// Automatically kills the server process and cleans up temp files on drop.
//
// // 表示一个用于测试的运行中的 TurtleShare 服务器实例。
// // 在 Drop 时自动终止服务器进程并清理临时文件。
pub struct TestServer {
    /// HTTP client pre-configured with the server's base URL.
    // // 预配置了服务器基础 URL 的 HTTP 客户端。
    pub client: reqwest::Client,

    /// Base URL of the running server (e.g., "http://127.0.0.1:12345").
    // // 运行中服务器的基础 URL。
    pub base_url: String,

    /// Port the server is listening on.
    // // 服务器监听的端口。
    pub port: u16,

    /// Path to the temporary database file.
    // // 临时数据库文件路径。
    pub db_path: PathBuf,

    /// Path to the temporary files storage directory.
    // // 临时文件存储目录路径。
    pub files_path: PathBuf,

    /// The server child process handle.
    // // 服务器子进程句柄。
    process: Child,

    /// Temp directory handle - kept alive to prevent premature cleanup.
    // // 临时目录句柄 - 保持存活以防止过早清理。
    _temp_dir: TempDir,
}

impl TestServer {
    /// Spawns a new TurtleShare server in an isolated environment.
    ///
    /// Creates a temporary directory with a generated config.toml,
    /// finds a free port, starts the binary, and waits for it to be ready.
    ///
    /// # Returns
    /// A `TestServer` instance ready for HTTP requests.
    //
    // // 在隔离环境中启动一个新的 TurtleShare 服务器。
    // // 创建临时目录和配置文件，找到空闲端口，启动二进制程序，等待就绪。
    pub async fn spawn() -> Self {
        Self::spawn_with_config(TestConfig::default()).await
    }

    /// Spawns a server with custom configuration overrides.
    //
    // // 使用自定义配置覆盖启动服务器。
    pub async fn spawn_with_config(config: TestConfig) -> Self {
        // 1. 创建临时目录结构
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        let db_path = temp_path.join("test.db");
        let files_path = temp_path.join("files");
        let static_path = temp_path.join("static");
        std::fs::create_dir_all(&files_path).unwrap();
        std::fs::create_dir_all(&static_path).unwrap();

        // 2. 找到一个空闲端口
        let port = find_free_port();

        // 3. 生成测试用配置文件
        let config_path = temp_path.join("config.toml");
        let config_content = config.to_toml(port, &db_path, &files_path, &static_path);
        let mut f = std::fs::File::create(&config_path).unwrap();
        f.write_all(config_content.as_bytes()).unwrap();

        // 4. 找到编译好的二进制文件
        let binary = cargo_bin_path();

        // 5. 启动服务器进程
        let process = Command::new(&binary)
            .arg("--config")
            .arg(&config_path)
            .env("RUST_LOG", "warn")
            .spawn()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to start TurtleShare binary at {:?}: {}",
                    binary, e
                )
            });

        let base_url = format!("http://127.0.0.1:{}", port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        let server = TestServer {
            client,
            base_url,
            port,
            db_path,
            files_path,
            process,
            _temp_dir: temp_dir,
        };

        // 6. 等待服务器就绪
        server.wait_until_ready().await;

        server
    }

    /// Constructs a full URL for the given API path.
    ///
    /// # Example
    /// ```
    /// let url = server.url("/api/health");
    /// // => "http://127.0.0.1:12345/api/health"
    /// ```
    //
    // // 为给定的 API 路径构造完整 URL。
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Sends a GET request to the given path and returns the response.
    //
    // // 向给定路径发送 GET 请求并返回响应。
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .send()
            .await
            .expect("GET request failed")
    }

    /// Sends a POST request with JSON body to the given path.
    //
    // // 向给定路径发送带 JSON 请求体的 POST 请求。
    pub async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .expect("POST request failed")
    }

    /// Sends a PUT request with JSON body to the given path.
    //
    // // 向给定路径发送带 JSON 请求体的 PUT 请求。
    pub async fn put_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> reqwest::Response {
        self.client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .expect("PUT request failed")
    }

    /// Sends a DELETE request to the given path.
    //
    // // 向给定路径发送 DELETE 请求。
    pub async fn delete(&self, path: &str) -> reqwest::Response {
        self.client
            .delete(self.url(path))
            .send()
            .await
            .expect("DELETE request failed")
    }

    /// Sends a GET request with an Authorization Bearer token.
    //
    // // 发送带 Authorization Bearer 令牌的 GET 请求。
    pub async fn get_with_token(
        &self,
        path: &str,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Authenticated GET request failed")
    }

    /// Sends a POST request with JSON body and Bearer token.
    //
    // // 发送带 JSON 请求体和 Bearer 令牌的 POST 请求。
    pub async fn post_json_with_token(
        &self,
        path: &str,
        body: &serde_json::Value,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .expect("Authenticated POST request failed")
    }

    /// Sends a PUT request with JSON body and Bearer token.
    //
    // // 发送带 JSON 请求体和 Bearer 令牌的 PUT 请求。
    pub async fn put_json_with_token(
        &self,
        path: &str,
        body: &serde_json::Value,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .put(self.url(path))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .expect("Authenticated PUT request failed")
    }

    /// Sends a DELETE request with Bearer token.
    //
    // // 发送带 Bearer 令牌的 DELETE 请求。
    pub async fn delete_with_token(
        &self,
        path: &str,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .delete(self.url(path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Authenticated DELETE request failed")
    }

    /// Sends a POST request with multipart form data and Bearer token.
    //
    // // 发送带 multipart 表单数据和 Bearer 令牌的 POST 请求。
    pub async fn post_multipart_with_token(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
        token: &str,
    ) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await
            .expect("Authenticated multipart POST request failed")
    }

    /// Logs in as admin and returns the JWT token.
    //
    // // 以管理员身份登录并返回 JWT 令牌。
    pub async fn admin_login(&self) -> String {
        self.admin_login_with("admin", "admin123").await
    }

    /// Logs in as admin with custom credentials and returns the JWT token.
    //
    // // 使用自定义凭据以管理员身份登录并返回 JWT 令牌。
    pub async fn admin_login_with(
        &self,
        username: &str,
        password: &str,
    ) -> String {
        let resp = self
            .post_json(
                "/api/admin/login",
                &serde_json::json!({
                    "username": username,
                    "password": password
                }),
            )
            .await;

        assert_eq!(resp.status(), 200, "Admin login failed");
        let body: serde_json::Value = resp.json().await.unwrap();
        body["data"]["token"]
            .as_str()
            .expect("No token in login response")
            .to_string()
    }

    /// Polls the health endpoint until the server is ready or timeout.
    //
    // // 轮询健康检查端点，直到服务器就绪或超时。
    async fn wait_until_ready(&self) {
        let max_wait = Duration::from_secs(15);
        let poll_interval = Duration::from_millis(100);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > max_wait {
                panic!(
                    "Server did not become ready within {:?} at {}",
                    max_wait, self.base_url
                );
            }

            match self.client.get(self.url("/api/health")).send().await {
                Ok(resp) if resp.status().is_success() => return,
                _ => tokio::time::sleep(poll_interval).await,
            }
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // 终止服务器进程
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Configuration overrides for test servers.
//
// // 测试服务器的配置覆盖。
pub struct TestConfig {
    pub admin_username: String,
    pub admin_password_hash: String,
    pub jwt_base_secret: String,
    pub jwt_expiry_hours: u64,
    pub jwt_rotation_days: u64,
    pub hashid_min_length: usize,
    pub max_upload_size_mb: u64,
    /// Raw TOML content for the `[siteinfo]` section.
    /// If `None`, a default section with `name` and `author` is generated.
    //
    // // `[siteinfo]` 部分的原始 TOML 内容。
    // // 如果为 `None`，则生成包含 `name` 和 `author` 的默认部分。
    pub siteinfo_toml: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            admin_username: "admin".to_string(),
            // admin123 的 argon2id 哈希
            admin_password_hash: "$argon2id$v=19$m=19456,t=2,p=1$FVGhfDUHIQpSCUabKbhkVA$e0tpWtkmWL7uKmX2t517HOAHpUuBbmIpluFwDv522Ns".to_string(),
            jwt_base_secret: "test-secret-key-for-integration-tests".to_string(),
            jwt_expiry_hours: 24,
            jwt_rotation_days: 30,
            hashid_min_length: 6,
            max_upload_size_mb: 10,
            siteinfo_toml: None,
        }
    }
}

impl TestConfig {
    /// Renders this config as a TOML string with the given runtime values.
    //
    // // 使用给定的运行时值将此配置渲染为 TOML 字符串。
    fn to_toml(
        &self,
        port: u16,
        db_path: &std::path::Path,
        files_path: &std::path::Path,
        static_path: &std::path::Path,
    ) -> String {
        // 将路径中的反斜杠转换为正斜杠（TOML 兼容）
        let db_str = db_path.to_str().unwrap().replace('\\', "/");
        let files_str = files_path.to_str().unwrap().replace('\\', "/");
        let static_str = static_path.to_str().unwrap().replace('\\', "/");

        // 使用自定义 siteinfo 或默认值
        let siteinfo_section = match &self.siteinfo_toml {
            Some(custom) => custom.clone(),
            None => format!(
                r#"[siteinfo]
name = "TurtleShare-Test"
author = "TestAdmin"
sponsor_link = ""
header_image = ""
"#,
            ),
        };

        format!(
            r#"[admin]
username = "{admin_username}"
password_hash = "{admin_password_hash}"

[server]
host = "127.0.0.1"
port = {port}
base_url = "http://127.0.0.1:{port}"

[database]
path = "{db_path}"

[storage]
static_path = "{static_path}"
files_path = "{files_path}"
max_upload_size_mb = {max_upload_size_mb}

[jwt]
base_secret = "{jwt_base_secret}"
expiry_hours = {jwt_expiry_hours}
rotation_days = {jwt_rotation_days}

[hashid]
min_length = {hashid_min_length}

{siteinfo_section}
"#,
            admin_username = self.admin_username,
            admin_password_hash = self.admin_password_hash,
            port = port,
            db_path = db_str,
            static_path = static_str,
            files_path = files_str,
            max_upload_size_mb = self.max_upload_size_mb,
            jwt_base_secret = self.jwt_base_secret,
            jwt_expiry_hours = self.jwt_expiry_hours,
            jwt_rotation_days = self.jwt_rotation_days,
            hashid_min_length = self.hashid_min_length,
            siteinfo_section = siteinfo_section,
        )
    }
}

/// Finds a free TCP port by binding to port 0.
//
// // 通过绑定到端口 0 来找到一个空闲的 TCP 端口。
fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to free port");
    listener.local_addr().unwrap().port()
}

/// Returns the path to the compiled TurtleShare binary.
//
// // 返回编译好的 TurtleShare 二进制文件路径。
fn cargo_bin_path() -> PathBuf {
    // cargo test 会设置这个环境变量指向 target/debug/deps 目录
    let mut path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("Failed to get parent dir")
        .parent()
        .expect("Failed to get target dir")
        .to_path_buf();
    path.push("TurtleShare.exe");

    if !path.exists() {
        // 尝试不带 .exe 后缀（Unix 系统）
        path.set_extension("");
    }

    assert!(
        path.exists(),
        "TurtleShare binary not found at {:?}. Run `cargo build` first.",
        path
    );

    path
}
