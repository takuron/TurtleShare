// File handlers - Admin file management CRUD
//
// // 文件处理器 - 管理员文件管理 CRUD

use super::common::ApiResponse;
use crate::error::AppError;
use crate::handlers::admin::AdminState;
use crate::models::file::FileMetadata;
use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// List all files.
///
/// Returns a list of all uploaded files ordered by created_at descending.
//
// // 列出所有文件。
// //
// // 返回按 created_at 降序排列的所有已上传文件列表。
pub async fn list_files(
    State(state): State<AdminState>,
) -> Result<impl IntoResponse, AppError> {
    // 查询所有文件，按创建时间降序排列
    let files = sqlx::query_as::<_, FileMetadata>(
        "SELECT id, uuid, original_name, file_size, created_at FROM files ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 转换为带有 hash_id 的响应
    let responses = files
        .iter()
        .map(|f| f.to_response(&state.hashid_manager, &state.base_url))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}

/// Get file metadata.
///
/// Retrieves metadata for a single file by hash_id.
//
// // 获取文件元数据。
// //
// // 通过 hash_id 检索单个文件的元数据。
pub async fn get_file(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询文件
    let file = sqlx::query_as::<_, FileMetadata>(
        "SELECT id, uuid, original_name, file_size, created_at FROM files WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: file.to_response(&state.hashid_manager, &state.base_url)?,
    }))
}

/// Upload a file.
///
/// Accepts a multipart form upload. The file is stored under a UUID v4 directory
/// with its original filename. File size is validated against max_upload_size_mb.
///
/// # Arguments
/// * Multipart form with a `file` field
///
/// # Returns
/// The created file metadata with hash_id and access URL.
///
/// # Errors
/// Returns `VALIDATION_ERROR` if no file is provided or file exceeds size limit.
//
// // 上传文件。
// //
// // 接受 multipart 表单上传。文件存储在 UUID v4 目录下，保留原始文件名。
// // 文件大小根据 max_upload_size_mb 进行验证。
// //
// // # 参数
// // * 包含 `file` 字段的 Multipart 表单
// //
// // # 返回
// // 创建的文件元数据，包含 hash_id 和访问 URL。
// //
// // # 错误
// // 如果未提供文件或文件超过大小限制，返回 `VALIDATION_ERROR`。
pub async fn upload_file(
    State(state): State<AdminState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 multipart 表单中提取文件字段
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ValidationError(format!("Invalid multipart data: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            // 获取原始文件名
            let original_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unnamed".to_string());

            // 读取文件内容
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::ValidationError(format!("Failed to read file: {}", e)))?;

            file_data = Some((original_name, data.to_vec()));
            break;
        }
    }

    let (original_name, data) = file_data
        .ok_or_else(|| AppError::ValidationError("No file field provided".to_string()))?;

    // 2. 验证文件大小
    let file_size = data.len() as u64;
    if file_size > state.max_upload_size_bytes {
        let max_mb = state.max_upload_size_bytes / (1024 * 1024);
        return Err(AppError::ValidationError(
            format!("File size exceeds maximum allowed size of {} MB", max_mb),
        ));
    }

    // 3. 生成 UUID v4 作为文件目录名
    let file_uuid = Uuid::new_v4().to_string();

    // 4. 创建目录并写入文件
    let dir_path = std::path::Path::new(&state.files_path).join(&file_uuid);
    tokio::fs::create_dir_all(&dir_path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create directory: {}", e)))?;

    let file_path = dir_path.join(&original_name);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

    // 5. 插入数据库记录
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let id = sqlx::query(
        "INSERT INTO files (uuid, original_name, file_size, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&file_uuid)
    .bind(&original_name)
    .bind(file_size as i64)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .last_insert_rowid();

    let file_meta = FileMetadata {
        id,
        uuid: file_uuid,
        original_name,
        file_size: file_size as i64,
        created_at: now,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: file_meta.to_response(&state.hashid_manager, &state.base_url)?,
        }),
    ))
}

/// Delete a file.
///
/// Removes a file from the database and deletes it from disk.
///
/// # Arguments
/// * `hash_id` - The file's hash ID
///
/// # Errors
/// Returns `NOT_FOUND` if the file does not exist.
//
// // 删除文件。
// //
// // 从数据库中移除文件记录并从磁盘删除文件。
// //
// // # 参数
// // * `hash_id` - 文件的哈希 ID
// //
// // # 错误
// // 如果文件不存在，返回 `NOT_FOUND`。
pub async fn delete_file(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询文件记录（需要 uuid 来删除磁盘文件）
    let file = sqlx::query_as::<_, FileMetadata>(
        "SELECT id, uuid, original_name, file_size, created_at FROM files WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // 3. 删除数据库记录
    sqlx::query("DELETE FROM files WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. 删除磁盘上的文件目录（uuid 目录及其内容）
    let dir_path = std::path::Path::new(&state.files_path).join(&file.uuid);
    if dir_path.exists() {
        tokio::fs::remove_dir_all(&dir_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete file: {}", e)))?;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({
            "deleted": true,
            "hash_id": hash_id
        }),
    }))
}
