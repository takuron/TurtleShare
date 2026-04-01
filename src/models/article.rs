use crate::error::AppError;
use crate::utils::hashid::HashIdManager;
use serde::{Deserialize, Serialize};

/// A single file link entry within an article.
///
/// Represents a downloadable file associated with the article,
/// containing a display name and an absolute URL.
//
// // 文章中的单个文件链接条目。
// //
// // 表示与文章关联的可下载文件，包含显示名称和绝对链接。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileLink {
    /// Display name of the file / 文件显示名称
    pub name: String,
    /// Absolute URL to the file / 文件的绝对链接
    pub url: String,
}

/// Article model representing a content article in the database.
///
/// Maps directly to the `articles` table. The `is_public` field is stored
/// as INTEGER in SQLite but represented as bool in Rust.
/// The `file_links` field is stored as a JSON string in the database.
//
// // 文章模型，表示数据库中的内容文章。
// //
// // 直接映射到 `articles` 表。`is_public` 字段在 SQLite 中存储为 INTEGER，
// // 但在 Rust 中表示为 bool。
// // `file_links` 字段在数据库中存储为 JSON 字符串。
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub cover_image: Option<String>,
    pub content: String,
    pub required_tier: i32,
    pub is_public: bool,
    /// Stored as JSON string in DB, parsed to Vec<FileLink> in API responses.
    /// / 在数据库中存储为 JSON 字符串，在 API 响应中解析为 Vec<FileLink>。
    pub file_links: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Article response with hash_id for API responses.
///
/// Exposes article data with hash_id instead of the raw numeric id.
/// The `file_links` field is deserialized from JSON string to structured array.
//
// // 带有 hash_id 的文章响应，用于 API 响应。
// //
// // 暴露文章数据时使用 hash_id 代替原始数字 id。
// // `file_links` 字段从 JSON 字符串反序列化为结构化数组。
#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub hash_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub content: String,
    pub required_tier: i32,
    pub is_public: bool,
    /// Structured file links array / 结构化文件链接数组
    pub file_links: Vec<FileLink>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Article {
    /// Parses the stored JSON string into a Vec<FileLink>.
    ///
    /// Returns an empty Vec if file_links is None or empty.
    //
    // // 将存储的 JSON 字符串解析为 Vec<FileLink>。
    // //
    // // 如果 file_links 为 None 或空，返回空 Vec。
    pub fn parse_file_links(&self) -> Result<Vec<FileLink>, AppError> {
        match &self.file_links {
            None => Ok(Vec::new()),
            Some(s) if s.is_empty() => Ok(Vec::new()),
            Some(s) => serde_json::from_str(s)
                .map_err(|e| AppError::Internal(format!("Invalid file_links JSON in DB: {}", e))),
        }
    }

    /// Converts Article to ArticleResponse with encoded hash_id.
    //
    // // 将 Article 转换为带有编码 hash_id 的 ArticleResponse。
    pub fn to_response(&self, hash_id_manager: &HashIdManager) -> Result<ArticleResponse, AppError> {
        Ok(ArticleResponse {
            hash_id: hash_id_manager.encode(self.id)?,
            title: self.title.clone(),
            cover_image: self.cover_image.clone(),
            content: self.content.clone(),
            required_tier: self.required_tier,
            is_public: self.is_public,
            file_links: self.parse_file_links()?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Serializes a Vec<FileLink> to a JSON string for DB storage.
///
/// Returns None if the vec is empty.
//
// // 将 Vec<FileLink> 序列化为 JSON 字符串以存储到数据库。
// //
// // 如果 vec 为空，返回 None。
pub fn serialize_file_links(links: &[FileLink]) -> Option<String> {
    if links.is_empty() {
        None
    } else {
        Some(serde_json::to_string(links).unwrap())
    }
}

/// Request payload for creating an article.
//
// // 创建文章的请求载荷。
#[derive(Debug, Deserialize)]
pub struct CreateArticleRequest {
    /// Article title / 文章标题
    pub title: String,
    /// Optional cover image path / 可选的封面图片路径
    pub cover_image: Option<String>,
    /// Article content (Markdown) / 文章内容（Markdown）
    pub content: String,
    /// Minimum subscription tier required to access (0-255) / 访问所需的最低订阅等级（0-255）
    pub required_tier: i32,
    /// Whether the article is publicly listed / 文章是否公开列出
    pub is_public: bool,
    /// File links array, each with name and absolute url / 文件链接数组，每个包含名称和绝对链接
    #[serde(default)]
    pub file_links: Vec<FileLink>,
}

/// Request payload for updating an existing article.
///
/// All fields are optional; only provided fields will be updated.
//
// // 更新现有文章的请求载荷。
// //
// // 所有字段都是可选的；仅更新提供的字段。
#[derive(Debug, Deserialize)]
pub struct UpdateArticleRequest {
    /// Optional new title / 可选的新标题
    pub title: Option<String>,
    /// Optional new cover image path / 可选的新封面图片路径
    pub cover_image: Option<String>,
    /// Optional new content / 可选的新内容
    pub content: Option<String>,
    /// Optional new required tier / 可选的新所需等级
    pub required_tier: Option<i32>,
    /// Optional new public status / 可选的新公开状态
    pub is_public: Option<bool>,
    /// Optional new file links array / 可选的新文件链接数组
    pub file_links: Option<Vec<FileLink>>,
}
