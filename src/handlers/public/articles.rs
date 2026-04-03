// Public articles handler - No authentication required
//
// // 公开文章处理器 - 无需身份验证

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Serialize;

use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::article::Article;
use crate::utils::hashid::HashIdManager;
use sqlx::SqlitePool;
use std::sync::Arc;

/// State for public article handlers.
///
/// Contains shared resources needed for public article operations.
//
// // 公开文章处理器的状态。
// //
// // 包含公开文章操作所需的共享资源。
#[derive(Clone)]
pub struct PublicArticleState {
    pub pool: SqlitePool,
    pub hashid_manager: Arc<HashIdManager>,
}

/// Article list item for public article list.
///
/// Does not include content, is_public, and file_links fields.
/// Includes accessible field to indicate if the content can be fully accessed.
//
// // 公开文章列表的文章列表项。
// //
// // 不包含 content、is_public 和 file_links 字段。
// // 包含 accessible 字段以指示是否可以完整访问内容。
#[derive(Debug, Serialize)]
pub struct PublicArticleListItem {
    pub hash_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub required_tier: i32,
    /// Whether the article content can be fully accessed.
    /// For public users, this is true only when required_tier = 0.
    /// / 是否可以完整访问文章内容。
    /// 对于公开用户，仅当 required_tier = 0 时为 true。
    pub accessible: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Article {
    /// Converts Article to PublicArticleListItem with accessibility check.
    ///
    /// For public (unauthenticated) users, accessible is true only when required_tier = 0.
    ///
    /// # Arguments
    /// * `hash_id_manager` - The HashID manager for encoding IDs
    ///
    /// # Returns
    /// Returns the PublicArticleListItem with accessibility determined by tier.
    //
    // // 将 Article 转换为带有可访问性检查的 PublicArticleListItem。
    // //
    // // 对于公开（未认证）用户，仅当 required_tier = 0 时 accessible 为 true。
    // //
    // // # 参数
    // // * `hash_id_manager` - 用于编码 ID 的 HashID 管理器
    // //
    // // # 返回
    // // 返回通过等级确定可访问性的 PublicArticleListItem。
    fn to_public_list_item(
        &self,
        hash_id_manager: &HashIdManager,
    ) -> Result<PublicArticleListItem, AppError> {
        // 公开用户的等级视为 0，只有 tier=0 的文章可完整访问
        let accessible = self.required_tier == 0;

        Ok(PublicArticleListItem {
            hash_id: hash_id_manager.encode(self.id)?,
            title: self.title.clone(),
            cover_image: self.cover_image.clone(),
            required_tier: self.required_tier,
            accessible,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// List public articles handler.
///
/// Returns a list of articles that are publicly listed (is_public = true).
/// The accessible field indicates if the content can be fully accessed:
/// - accessible = true: required_tier = 0, full content available via detail endpoint
/// - accessible = false: required_tier > 0, detail endpoint will return 403
///
/// The content, is_public, and file_links fields are excluded from the response.
///
/// # Arguments
/// * `state` - Application state containing database pool and HashID manager
///
/// # Returns
/// Returns a list of public articles ordered by created_at descending.
///
/// # Errors
/// Returns `Database` error on database failures.
//
// // 公开文章列表处理器。
// //
// // 返回公开列出的文章列表（is_public = true）。
// // accessible 字段指示是否可以完整访问内容：
// // - accessible = true：required_tier = 0，可通过详情端点获取完整内容
// // - accessible = false：required_tier > 0，详情端点将返回 403
// //
// // 响应中不包含 content、is_public 和 file_links 字段。
// //
// // # 参数
// // * `state` - 包含数据库连接池和 HashID 管理器的应用状态
// //
// // # 返回
// // 返回按 created_at 降序排列的公开文章列表。
// //
// // # 错误
// // 数据库失败时返回 `Database` 错误。
pub async fn list_articles(
    State(state): State<PublicArticleState>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 查询所有公开文章（is_public = true）
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at
         FROM articles
         WHERE is_public = 1
         ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 2. 转换为公开列表项
    let items: Result<Vec<PublicArticleListItem>, AppError> = articles
        .iter()
        .map(|article| article.to_public_list_item(&state.hashid_manager))
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: items?,
    }))
}

/// Get public article detail handler.
///
/// Retrieves a single article by hash_id if it is publicly accessible.
/// Returns full article content only if required_tier = 0 (accessible = true).
/// Returns 403 Forbidden if required_tier > 0 (accessible = false).
///
/// # Arguments
/// * `state` - Application state containing database pool and HashID manager
/// * `hash_id` - The hash ID of the article to retrieve
///
/// # Returns
/// Returns the full article details if accessible.
///
/// # Errors
/// Returns `NotFound` if article does not exist or is not public.
/// Returns `Forbidden` if article requires subscription (required_tier > 0).
/// Returns `Database` error on database failures.
//
// // 获取公开文章详情处理器。
// //
// // 通过 hash_id 检索单篇文章（如果可公开访问）。
// // 仅当 required_tier = 0（accessible = true）时返回完整文章内容。
// // 如果 required_tier > 0（accessible = false），返回 403 Forbidden。
// //
// // # 参数
// // * `state` - 包含数据库连接池和 HashID 管理器的应用状态
// // * `hash_id` - 要检索的文章的 hash ID
// //
// // # 返回
// // 如果可访问，返回完整的文章详情。
// //
// // # 错误
// // 如果文章不存在或不是公开的，返回 `NotFound`。
// // 如果文章需要订阅（required_tier > 0），返回 `Forbidden`。
// // 数据库失败时返回 `Database` 错误。
pub async fn get_article(
    State(state): State<PublicArticleState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码文章 hash_id 为数字 ID
    let article_id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询文章
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at
         FROM articles
         WHERE id = ?"
    )
    .bind(article_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    // 3. 检查文章是否公开（is_public = true）
    if !article.is_public {
        return Err(AppError::NotFound("Article not found".to_string()));
    }

    // 4. 检查访问权限：公开用户等级视为 0，只有 tier=0 的文章可访问
    if article.required_tier > 0 {
        return Err(AppError::Forbidden(
            "Insufficient subscription tier to access this article".to_string(),
        ));
    }

    // 5. 返回文章完整详情
    Ok(Json(ApiResponse {
        success: true,
        data: article.to_client_detail_response(&state.hashid_manager)?,
    }))
}
