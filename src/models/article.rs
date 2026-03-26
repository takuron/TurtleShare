use serde::{Deserialize, Serialize};

/// Article model representing a content article.
//
// // 文章模型，表示内容文章。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub cover_image: Option<String>,
    pub content: String,
    pub required_tier: i32,
    pub is_public: bool,
    pub file_links: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request payload for creating an article.
//
// // 创建文章请求载荷。
#[derive(Debug, Deserialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub cover_image: Option<String>,
    pub content: String,
    pub required_tier: i32,
    pub is_public: bool,
    pub file_links: Option<String>,
}
