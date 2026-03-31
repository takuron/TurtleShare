use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;
use crate::utils::hashid::HashIdManager;

/// File metadata model stored in the database.
///
/// Represents a file record with UUID-based storage path.
///
/// # Fields
/// * `id` - Auto-incremented database ID (never exposed)
/// * `uuid` - UUID v4 used as the file's directory name
/// * `original_name` - Original filename from upload
/// * `file_size` - File size in bytes
/// * `created_at` - Unix timestamp of creation
//
// // 存储在数据库中的文件元数据模型。
// //
// // 表示一个基于 UUID 存储路径的文件记录。
// //
// // # 字段
// // * `id` - 自增数据库 ID（永不暴露）
// // * `uuid` - 用作文件目录名的 UUID v4
// // * `original_name` - 上传时的原始文件名
// // * `file_size` - 文件大小（字节）
// // * `created_at` - 创建时间的 Unix 时间戳
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct FileMetadata {
    pub id: i64,
    pub uuid: String,
    pub original_name: String,
    pub file_size: i64,
    pub created_at: i64,
}

/// File metadata response for API output.
///
/// Uses hash_id instead of numeric id. Includes a URL for file access.
//
// // API 输出的文件元数据响应。
// //
// // 使用 hash_id 代替数字 id。包含文件访问 URL。
#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub hash_id: String,
    pub uuid: String,
    pub original_name: String,
    pub file_size: i64,
    pub url: String,
    pub created_at: i64,
}

impl FileMetadata {
    /// Converts the database model to an API response with hash_id and URL.
    ///
    /// # Arguments
    /// * `hashid_manager` - HashID manager for encoding the ID
    /// * `base_url` - Server base URL for constructing file access URL
    ///
    /// # Returns
    /// A `FileResponse` with the encoded hash_id and full file URL.
    //
    // // 将数据库模型转换为带有 hash_id 和 URL 的 API 响应。
    // //
    // // # 参数
    // // * `hashid_manager` - 用于编码 ID 的 HashID 管理器
    // // * `base_url` - 用于构建文件访问 URL 的服务器基础 URL
    // //
    // // # 返回
    // // 带有编码 hash_id 和完整文件 URL 的 `FileResponse`。
    pub fn to_response(
        &self,
        hashid_manager: &HashIdManager,
        base_url: &str,
    ) -> Result<FileResponse, AppError> {
        let hash_id = hashid_manager.encode(self.id)?;
        // 构建文件访问 URL: {base_url}/files/{uuid}/{original_name}
        let url = format!(
            "{}/files/{}/{}",
            base_url.trim_end_matches('/'),
            self.uuid,
            self.original_name
        );
        Ok(FileResponse {
            hash_id,
            uuid: self.uuid.clone(),
            original_name: self.original_name.clone(),
            file_size: self.file_size,
            url,
            created_at: self.created_at,
        })
    }
}
