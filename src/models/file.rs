use serde::{Deserialize, Serialize};

/// File metadata model.
//
// // 文件元数据模型。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub id: i64,
    pub uuid: String,
    pub original_name: String,
    pub file_size: i64,
    pub created_at: i64,
}
