// Announcement model - Stored as JSON in kv_store
//
// // 公告模型 - 以 JSON 格式存储在 kv_store 中

use serde::{Deserialize, Serialize};

/// Announcement data stored in kv_store under the key "announcement".
///
/// Contains the announcement content and the timestamp of the last update.
/// The entire struct is serialized as a JSON string and stored in the `value` column.
//
// // 存储在 kv_store 中键为 "announcement" 的公告数据。
// //
// // 包含公告内容和最后更新时间戳。
// // 整个结构体序列化为 JSON 字符串并存储在 `value` 列中。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementData {
    /// The announcement content (Markdown).
    //
    // // 公告内容（Markdown）。
    pub content: String,

    /// Unix timestamp of the last update.
    //
    // // 最后更新的 Unix 时间戳。
    pub updated_at: i64,
}

/// Request body for publishing an announcement.
//
// // 发布公告的请求体。
#[derive(Debug, Deserialize)]
pub struct PublishAnnouncementRequest {
    /// The announcement content (Markdown). Empty or whitespace-only content deletes the announcement.
    //
    // // 公告内容（Markdown）。内容为空或仅含空白字符时删除公告。
    pub content: String,
}
