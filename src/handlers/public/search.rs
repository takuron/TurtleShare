// Public search handler - No authentication required
//
// // 公开搜索处理器 - 无需身份验证

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::article::Article;
use crate::models::user::User;
use crate::utils::hashid::HashIdManager;
use crate::handlers::public::PublicState;

/// Search query parameters.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(rename = "type")]
    pub search_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Search type enumeration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchType {
    All,
    Articles,
    Users,
}

impl Default for SearchType {
    fn default() -> Self {
        SearchType::All
    }
}

impl SearchType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "articles" => SearchType::Articles,
            "users" => SearchType::Users,
            _ => SearchType::All,
        }
    }
}

/// Public user search result item.
/// Only includes non-sensitive information.
#[derive(Debug, Serialize)]
pub struct PublicUserSearchItem {
    pub hash_id: String,
    pub username: String,
}

/// Public article search result item.
#[derive(Debug, Serialize)]
pub struct PublicArticleSearchItem {
    pub hash_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub required_tier: i32,
    pub accessible: bool,
    pub publish_at: i64,
    pub updated_at: i64,
}

/// Combined search response.
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub articles: Vec<PublicArticleSearchItem>,
    pub users: Vec<PublicUserSearchItem>,
    pub total_articles: u32,
    pub total_users: u32,
}

impl Article {
    fn to_search_item(
        &self,
        hash_id_manager: &HashIdManager,
    ) -> Result<PublicArticleSearchItem, AppError> {
        let accessible = self.required_tier == 0;

        Ok(PublicArticleSearchItem {
            hash_id: hash_id_manager.encode(self.id)?,
            title: self.title.clone(),
            cover_image: self.cover_image.clone(),
            required_tier: self.required_tier,
            accessible,
            publish_at: self.publish_at,
            updated_at: self.updated_at,
        })
    }
}

impl User {
    fn to_search_item(
        &self,
        hash_id_manager: &HashIdManager,
    ) -> Result<PublicUserSearchItem, AppError> {
        Ok(PublicUserSearchItem {
            hash_id: hash_id_manager.encode(self.id)?,
            username: self.username.clone(),
        })
    }
}

/// Search handler.
///
/// Searches for articles and users based on the query string.
/// - Articles: searched by title (LIKE %q%)
/// - Users: searched by username (LIKE %q%)
///
/// # Query Parameters
/// - `q`: Search keyword (required)
/// - `type`: Search type ("all", "articles", "users"), default: "all"
/// - `page`: Page number (1-based), default: 1
/// - `page_size`: Items per page, default: 10
///
/// # Returns
/// Returns search results with articles and users.
pub async fn search(
    State(state): State<PublicState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let keyword = query.q.trim();
    if keyword.is_empty() {
        return Ok(Json(ApiResponse {
            success: true,
            data: SearchResponse {
                articles: Vec::new(),
                users: Vec::new(),
                total_articles: 0,
                total_users: 0,
            },
        }));
    }

    let search_type = query
        .search_type
        .map(|s| SearchType::from_str(&s))
        .unwrap_or_default();

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let like_pattern = format!("%{}%", keyword);

    let mut articles: Vec<PublicArticleSearchItem> = Vec::new();
    let mut users: Vec<PublicUserSearchItem> = Vec::new();
    let mut total_articles: u32 = 0;
    let mut total_users: u32 = 0;

    if search_type == SearchType::All || search_type == SearchType::Articles {
        let count_result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM articles WHERE is_public = 1 AND title LIKE ?",
        )
        .bind(&like_pattern)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        total_articles = count_result.0 as u32;

        if total_articles > 0 {
            let article_rows = sqlx::query_as::<_, Article>(
                "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
                 FROM articles
                 WHERE is_public = 1 AND title LIKE ?
                 ORDER BY publish_at DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(&like_pattern)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            articles = article_rows
                .iter()
                .map(|a| a.to_search_item(&state.hashid_manager))
                .collect::<Result<_, _>>()?;
        }
    }

    if search_type == SearchType::All || search_type == SearchType::Users {
        let count_result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE username LIKE ?")
                .bind(&like_pattern)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        total_users = count_result.0 as u32;

        if total_users > 0 {
            let user_rows = sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, email, note, created_at
                 FROM users
                 WHERE username LIKE ?
                 ORDER BY created_at DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(&like_pattern)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            users = user_rows
                .iter()
                .map(|u| u.to_search_item(&state.hashid_manager))
                .collect::<Result<_, _>>()?;
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: SearchResponse {
            articles,
            users,
            total_articles,
            total_users,
        },
    }))
}
