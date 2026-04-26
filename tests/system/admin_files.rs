// 管理员文件管理 API 集成测试
//
// 测试 /api/admin/files 和 /api/admin/files/:hash_id 端点的完整行为。
// 包含正常操作、边界条件、错误输入和安全测试。

use super::common;
use reqwest::multipart;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// 计算字节数据的 SHA256 十六进制摘要。
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

// ============================================================
// 辅助函数
// ============================================================

/// 上传一个测试文件并返回 (hash_id, uuid)。
async fn upload_test_file(
    server: &common::TestServer,
    token: &str,
    filename: &str,
    content: &[u8],
) -> (String, String) {
    let part = multipart::Part::bytes(content.to_vec())
        .file_name(filename.to_string())
        .mime_str("application/octet-stream")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, token)
        .await;

    assert_eq!(resp.status(), 201, "Failed to upload file {}", filename);
    let body: Value = resp.json().await.unwrap();
    let hash_id = body["data"]["hash_id"].as_str().unwrap().to_string();
    let uuid = body["data"]["uuid"].as_str().unwrap().to_string();
    (hash_id, uuid)
}

// ============================================================
// 上传文件
// ============================================================

/// 上传文件应返回 201 和正确的文件信息，且可通过静态路径下载并校验 SHA256。
#[tokio::test]
async fn upload_file_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let file_content = b"Hello, TurtleShare!";
    let expected_sha256 = sha256_hex(file_content);

    let part = multipart::Part::bytes(file_content.to_vec())
        .file_name("hello.txt".to_string())
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, &token)
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    let data = &body["data"];
    let hash_id = data["hash_id"].as_str().unwrap();
    assert!(!hash_id.is_empty(), "hash_id should not be empty");
    assert_eq!(data["original_name"], "hello.txt");
    assert_eq!(data["file_size"], file_content.len() as i64);

    // uuid 应为有效的 UUID v4 格式
    let uuid = data["uuid"].as_str().unwrap();
    assert_eq!(uuid.len(), 36);
    assert!(uuid.contains('-'));

    // url 应包含 uuid 和原始文件名
    let url = data["url"].as_str().unwrap();
    assert!(url.contains(uuid));
    assert!(url.contains("hello.txt"));

    // created_at 应为合理的 Unix 时间戳
    let created_at = data["created_at"].as_i64().unwrap();
    assert!(created_at > 1_700_000_000);

    // 响应中不应包含数字 ID
    assert!(data["id"].is_null());

    // 通过静态路径下载文件并校验内容和 SHA256
    let download_path = format!("/files/{}/hello.txt", uuid);
    let download_resp = server.get(&download_path).await;
    assert_eq!(download_resp.status(), 200);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), file_content);
    assert_eq!(sha256_hex(&downloaded), expected_sha256);
}

/// 上传二进制文件应成功，且可通过静态路径下载并校验 SHA256。
#[tokio::test]
async fn upload_binary_file() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 模拟一个小的二进制文件（PNG 文件头）
    let binary_content: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52,
    ];
    let expected_sha256 = sha256_hex(&binary_content);

    let part = multipart::Part::bytes(binary_content.clone())
        .file_name("test.png".to_string())
        .mime_str("image/png")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, &token)
        .await;

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["original_name"], "test.png");
    assert_eq!(body["data"]["file_size"], binary_content.len() as i64);

    // 通过静态路径下载并校验二进制内容和 SHA256
    let uuid = body["data"]["uuid"].as_str().unwrap();
    let download_resp = server.get(&format!("/files/{}/test.png", uuid)).await;
    assert_eq!(download_resp.status(), 200);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), binary_content.as_slice());
    assert_eq!(sha256_hex(&downloaded), expected_sha256);
}

/// 上传空文件应返回 400（空文件被拒绝）。
#[tokio::test]
async fn upload_empty_file() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let empty_content: Vec<u8> = vec![];

    let part = multipart::Part::bytes(empty_content.clone())
        .file_name("empty.txt".to_string())
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 超过大小限制的文件应返回 400。
#[tokio::test]
async fn upload_file_exceeds_size_limit() {
    // 使用 max_upload_size_mb = 1 的配置
    let config = common::TestConfig {
        max_upload_size_mb: 1,
        ..common::TestConfig::default()
    };
    let server = common::TestServer::spawn_with_config(config).await;
    let token = server.admin_login().await;

    // 创建一个超过 1MB 的文件
    let large_content = vec![0u8; 1024 * 1024 + 1];
    let part = multipart::Part::bytes(large_content)
        .file_name("large.bin".to_string())
        .mime_str("application/octet-stream")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// 未认证上传文件应返回 401。
#[tokio::test]
async fn upload_file_without_auth() {
    let server = common::TestServer::spawn().await;

    let part = multipart::Part::bytes(b"test".to_vec())
        .file_name("test.txt".to_string())
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .client
        .post(server.url("/api/admin/files"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// 每次上传应生成不同的 UUID。
#[tokio::test]
async fn upload_files_unique_uuids() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let (hash_id1, _) = upload_test_file(&server, &token, "file1.txt", b"content1").await;
    let (hash_id2, _) = upload_test_file(&server, &token, "file2.txt", b"content2").await;

    // 获取两个文件的详情
    let resp1 = server
        .get_with_token(&format!("/api/admin/files/{}", hash_id1), &token)
        .await;
    let body1: Value = resp1.json().await.unwrap();
    let uuid1 = body1["data"]["uuid"].as_str().unwrap().to_string();

    let resp2 = server
        .get_with_token(&format!("/api/admin/files/{}", hash_id2), &token)
        .await;
    let body2: Value = resp2.json().await.unwrap();
    let uuid2 = body2["data"]["uuid"].as_str().unwrap().to_string();

    assert_ne!(uuid1, uuid2, "Each upload should have a unique UUID");
}

// ============================================================
// 列出文件
// ============================================================

/// 无文件时应返回空列表。
#[tokio::test]
async fn list_files_empty() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server.get_with_token("/api/admin/files", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let files = body["data"].as_array().unwrap();
    assert!(files.is_empty());
}

/// 上传多个文件后应全部返回，按 created_at 降序。
#[tokio::test]
async fn list_files_returns_all_ordered() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    upload_test_file(&server, &token, "file_a.txt", b"aaa").await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    upload_test_file(&server, &token, "file_b.txt", b"bbb").await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    upload_test_file(&server, &token, "file_c.txt", b"ccc").await;

    let resp = server.get_with_token("/api/admin/files", &token).await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let files = body["data"].as_array().unwrap();
    assert_eq!(files.len(), 3);

    // 验证按 created_at 降序排列
    let created_ats: Vec<i64> = files
        .iter()
        .map(|f| f["created_at"].as_i64().unwrap())
        .collect();
    assert!(created_ats[0] >= created_ats[1]);
    assert!(created_ats[1] >= created_ats[2]);

    // 验证没有泄露数字 ID
    for file in files {
        assert!(file["id"].is_null());
        assert!(!file["hash_id"].as_str().unwrap().is_empty());
    }
}

/// 未认证列出文件应返回 401。
#[tokio::test]
async fn list_files_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.get("/api/admin/files").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 获取文件详情
// ============================================================

/// 获取文件详情应返回完整信息。
#[tokio::test]
async fn get_file_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let (hash_id, _) = upload_test_file(&server, &token, "detail.txt", b"detail content").await;

    let resp = server
        .get_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["hash_id"], hash_id);
    assert_eq!(body["data"]["original_name"], "detail.txt");
    assert_eq!(body["data"]["file_size"], 14); // "detail content".len()
    assert!(body["data"]["id"].is_null());
}

/// 不存在的文件应返回 404。
#[tokio::test]
async fn get_file_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    // 创建再删除，得到合法但无效的 hash_id
    let (hash_id, _) = upload_test_file(&server, &token, "ghost.txt", b"ghost").await;
    server
        .delete_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    let resp = server
        .get_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 hash_id 应返回 400。
#[tokio::test]
async fn get_file_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .get_with_token("/api/admin/files/INVALID!!!", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

// ============================================================
// 删除文件
// ============================================================

/// 删除文件应成功，并从列表、磁盘和静态路径中移除。
#[tokio::test]
async fn delete_file_success() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;
    let (hash_id, uuid) = upload_test_file(&server, &token, "to_delete.txt", b"delete me").await;

    // 删除前通过静态路径应能下载
    let pre_dl = server.get(&format!("/files/{}/to_delete.txt", uuid)).await;
    assert_eq!(pre_dl.status(), 200);

    let resp = server
        .delete_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["deleted"], true);
    assert_eq!(body["data"]["hash_id"], hash_id);

    // 验证文件已从列表中移除
    let list_resp = server.get_with_token("/api/admin/files", &token).await;
    let list_body: Value = list_resp.json().await.unwrap();
    assert!(list_body["data"].as_array().unwrap().is_empty());

    // 验证磁盘上的文件目录已删除
    let dir_path = server.files_path.join(&uuid);
    assert!(
        !dir_path.exists(),
        "File directory should be deleted from disk"
    );

    // 删除后通过静态路径应返回 404
    let post_dl = server.get(&format!("/files/{}/to_delete.txt", uuid)).await;
    assert_eq!(post_dl.status(), 404);
}

/// 删除不存在的文件应返回 404。
#[tokio::test]
async fn delete_file_not_found() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let (hash_id, _) = upload_test_file(&server, &token, "delete_twice.txt", b"data").await;
    server
        .delete_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    let resp = server
        .delete_with_token(&format!("/api/admin/files/{}", hash_id), &token)
        .await;

    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

/// 无效的 hash_id 删除应返回 400。
#[tokio::test]
async fn delete_file_invalid_hash_id() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let resp = server
        .delete_with_token("/api/admin/files/INVALID!!!", &token)
        .await;

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_HASH_ID");
}

/// 未认证删除文件应返回 401。
#[tokio::test]
async fn delete_file_without_auth() {
    let server = common::TestServer::spawn().await;

    let resp = server.delete("/api/admin/files/somehash").await;
    assert_eq!(resp.status(), 401);
}

// ============================================================
// 安全性测试
// ============================================================

/// 所有文件操作在无认证时都应返回 401。
#[tokio::test]
async fn all_file_routes_require_auth() {
    let server = common::TestServer::spawn().await;

    // GET /api/admin/files
    assert_eq!(server.get("/api/admin/files").await.status(), 401);

    // GET /api/admin/files/:hash_id
    assert_eq!(server.get("/api/admin/files/abc123").await.status(), 401);

    // POST /api/admin/files (multipart without auth)
    let part = multipart::Part::bytes(b"test".to_vec())
        .file_name("test.txt".to_string())
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", part);
    let resp = server
        .client
        .post(server.url("/api/admin/files"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // DELETE /api/admin/files/:hash_id
    assert_eq!(server.delete("/api/admin/files/abc123").await.status(), 401);
}

/// 文件名中包含路径遍历字符不应影响存储安全，下载内容应正确。
#[tokio::test]
async fn upload_file_path_traversal_filename() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let content = b"malicious";
    let expected_sha256 = sha256_hex(content);

    let malicious_name = "../../../etc/passwd";
    let part = multipart::Part::bytes(content.to_vec())
        .file_name(malicious_name.to_string())
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", part);

    let resp = server
        .post_multipart_with_token("/api/admin/files", form, &token)
        .await;

    // 应成功创建（路径组件被剥离，只保留文件名部分）
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();

    // 文件名应被清理为只有最后的文件名部分
    assert_eq!(body["data"]["original_name"], "passwd");

    // 文件应存储在 UUID 目录下，不会逃逸
    let uuid = body["data"]["uuid"].as_str().unwrap();
    let file_dir = server.files_path.join(uuid);
    assert!(file_dir.exists(), "File should be stored in UUID directory");

    // 不应在 files_path 之外创建文件
    let escaped_path = server
        .files_path
        .parent()
        .unwrap()
        .join("etc")
        .join("passwd");
    assert!(
        !escaped_path.exists(),
        "Path traversal should not escape storage directory"
    );

    // 通过静态路径下载清理后的文件名并校验 SHA256
    let download_resp = server.get(&format!("/files/{}/passwd", uuid)).await;
    assert_eq!(download_resp.status(), 200);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), content.as_slice());
    assert_eq!(sha256_hex(&downloaded), expected_sha256);
}

/// 上传同名文件应各自存储在不同的 UUID 目录中，下载内容各自正确。
#[tokio::test]
async fn upload_duplicate_filename() {
    let server = common::TestServer::spawn().await;
    let token = server.admin_login().await;

    let content1 = b"version1";
    let content2 = b"version2";
    let (hash_id1, uuid1) = upload_test_file(&server, &token, "same.txt", content1).await;
    let (hash_id2, uuid2) = upload_test_file(&server, &token, "same.txt", content2).await;

    assert_ne!(hash_id1, hash_id2);
    assert_ne!(uuid1, uuid2, "Each upload should have a unique UUID");

    // 通过静态路径分别下载两个同名文件，校验内容和 SHA256
    let dl1 = server.get(&format!("/files/{}/same.txt", uuid1)).await;
    assert_eq!(dl1.status(), 200);
    let bytes1 = dl1.bytes().await.unwrap();
    assert_eq!(bytes1.as_ref(), content1);
    assert_eq!(sha256_hex(&bytes1), sha256_hex(content1));

    let dl2 = server.get(&format!("/files/{}/same.txt", uuid2)).await;
    assert_eq!(dl2.status(), 200);
    let bytes2 = dl2.bytes().await.unwrap();
    assert_eq!(bytes2.as_ref(), content2);
    assert_eq!(sha256_hex(&bytes2), sha256_hex(content2));

    // 两个文件的 SHA256 应不同
    assert_ne!(sha256_hex(&bytes1), sha256_hex(&bytes2));
}
