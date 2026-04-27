// Admin article management handlers
//
// // 管理员文章管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::{ApiResponse, PageCountResponse, PaginationQuery, SearchQuery};
use crate::models::article::{
    Article, CreateArticleRequest, UpdateArticleRequest, serialize_file_links,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Article list item for admin-facing article list.
///
/// Does not include content and file_links fields, but retains is_public.
//
// // 管理员面向的文章列表项。
// //
// // 不包含 content 和 file_links 字段，但保留 is_public。
#[derive(Debug, Serialize)]
pub struct AdminArticleListItem {
    pub hash_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub required_tier: i32,
    pub is_public: bool,
    pub publish_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Article {
    /// Converts Article to AdminArticleListItem.
    ///
    /// # Arguments
    /// * `hash_id_manager` - The HashID manager for encoding IDs
    ///
    /// # Returns
    /// Returns the AdminArticleListItem.
    //
    // // 将 Article 转换为 AdminArticleListItem。
    // //
    // // # 参数
    // // * `hash_id_manager` - 用于编码 ID 的 HashID 管理器
    // //
    // // # 返回
    // // 返回 AdminArticleListItem。
    fn to_admin_list_item(
        &self,
        hash_id_manager: &crate::utils::hashid::HashIdManager,
    ) -> Result<AdminArticleListItem, AppError> {
        Ok(AdminArticleListItem {
            hash_id: hash_id_manager.encode(self.id)?,
            title: self.title.clone(),
            cover_image: self.cover_image.clone(),
            required_tier: self.required_tier,
            is_public: self.is_public,
            publish_at: self.publish_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// List all articles.
///
/// Returns a list of all articles ordered by publish_at descending.
/// Excludes content and file_links from the response.
//
// // 列出所有文章。
// //
// // 返回按 publish_at 降序排列的所有文章列表。
// // 响应中不包含 content 和 file_links。
pub async fn list_articles(State(state): State<AdminState>) -> Result<impl IntoResponse, AppError> {
    // 查询所有文章，按发布时间降序排列
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at FROM articles ORDER BY publish_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 转换为带有 hash_id 的响应
    let responses: Vec<AdminArticleListItem> = articles
        .iter()
        .map(|a| a.to_admin_list_item(&state.hashid_manager))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}

/// Get article detail.
///
/// Retrieves a single article by hash_id.
//
// // 获取文章详情。
// //
// // 通过 hash_id 检索单篇文章。
pub async fn get_article(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询文章
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at FROM articles WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: article.to_response(&state.hashid_manager)?,
    }))
}

/// Create article.
///
/// Creates a new article with the provided data.
//
// // 创建文章。
// //
// // 使用提供的数据创建新文章。
pub async fn create_article(
    State(state): State<AdminState>,
    Json(req): Json<CreateArticleRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 验证 required_tier 范围（0-255）
    if req.required_tier < 0 || req.required_tier > 255 {
        return Err(AppError::ValidationError(
            "required_tier must be between 0 and 255".to_string(),
        ));
    }

    // 2. 验证标题非空
    if req.title.trim().is_empty() {
        return Err(AppError::ValidationError(
            "title must not be empty".to_string(),
        ));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 3. 序列化 file_links 为 JSON 字符串存储
    let file_links_json = serialize_file_links(&req.file_links);

    // 4. 计算 publish_at：如果未提供或为负数，默认与 created_at 相同
    let publish_at = match req.publish_at {
        Some(ts) if ts >= 0 => ts,
        _ => now,
    };

    // 5. 插入文章记录
    let id = sqlx::query(
        "INSERT INTO articles (title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.title)
    .bind(&req.cover_image)
    .bind(&req.content)
    .bind(req.required_tier)
    .bind(req.is_public)
    .bind(&file_links_json)
    .bind(publish_at)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .last_insert_rowid();

    let article = Article {
        id,
        title: req.title,
        cover_image: req.cover_image,
        content: req.content,
        required_tier: req.required_tier,
        is_public: req.is_public,
        file_links: file_links_json,
        publish_at,
        created_at: now,
        updated_at: now,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: article.to_response(&state.hashid_manager)?,
        }),
    ))
}

/// Update article.
///
/// Updates an existing article's information. Only provided fields are updated.
//
// // 更新文章。
// //
// // 更新现有文章的信息。仅更新提供的字段。
pub async fn update_article(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Json(req): Json<UpdateArticleRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    // 2. 查询现有文章
    let mut article = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at FROM articles WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    // 3. 更新提供的字段
    if let Some(title) = req.title {
        if title.trim().is_empty() {
            return Err(AppError::ValidationError(
                "title must not be empty".to_string(),
            ));
        }
        article.title = title;
    }
    if let Some(cover_image) = req.cover_image {
        if cover_image.is_empty() {
            article.cover_image = None;
        } else {
            article.cover_image = Some(cover_image);
        }
    }
    if let Some(content) = req.content {
        article.content = content;
    }
    if let Some(required_tier) = req.required_tier {
        if required_tier < 0 || required_tier > 255 {
            return Err(AppError::ValidationError(
                "required_tier must be between 0 and 255".to_string(),
            ));
        }
        article.required_tier = required_tier;
    }
    if let Some(is_public) = req.is_public {
        article.is_public = is_public;
    }
    if let Some(file_links) = req.file_links {
        article.file_links = serialize_file_links(&file_links);
    }
    // 更新 publish_at：如果为负数，重置为 created_at
    if let Some(publish_at) = req.publish_at {
        if publish_at < 0 {
            article.publish_at = article.created_at;
        } else {
            article.publish_at = publish_at;
        }
    }

    // 4. 更新 updated_at 时间戳
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    article.updated_at = now;

    // 5. 更新数据库
    sqlx::query(
        "UPDATE articles SET title = ?, cover_image = ?, content = ?, required_tier = ?, is_public = ?, file_links = ?, publish_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&article.title)
    .bind(&article.cover_image)
    .bind(&article.content)
    .bind(article.required_tier)
    .bind(article.is_public)
    .bind(&article.file_links)
    .bind(article.publish_at)
    .bind(article.updated_at)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: article.to_response(&state.hashid_manager)?,
    }))
}

/// Delete article.
///
/// Removes an article from the database.
//
// // 删除文章。
// //
// // 从数据库中移除文章。
pub async fn delete_article(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    // 2. 删除文章
    let rows_affected = sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Article not found".to_string()));
    }

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({
            "deleted": true,
            "hash_id": hash_id
        }),
    }))
}

/// Get total pages for articles.
///
/// Returns the total number of pages and items based on page_size.
//
// // 获取文章总页数。
// //
// // 基于 page_size 返回总页数和总项目数。
pub async fn get_articles_page_count(
    State(state): State<AdminState>,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page_size = query.page_size.unwrap_or(20).max(1);

    let total_items: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total_items = total_items.0 as u32;
    let total_pages = (total_items + page_size - 1) / page_size;

    Ok(Json(ApiResponse {
        success: true,
        data: PageCountResponse {
            total_pages,
            total_items,
        },
    }))
}

/// List articles paginated.
///
/// Returns a specific page of articles based on page and page_size.
/// Excludes content and file_links from the response.
//
// // 分页列出文章。
// //
// // 基于 page 和 page_size 返回特定页的文章。
// // 响应中不包含 content 和 file_links。
pub async fn list_articles_paginated(
    State(state): State<AdminState>,
    Path(page): Path<u32>,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page_size = query.page_size.unwrap_or(20).max(1);
    let page = page.max(1);
    let offset = (page - 1) * page_size;

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at FROM articles ORDER BY publish_at DESC LIMIT ? OFFSET ?"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let responses: Vec<AdminArticleListItem> = articles
        .iter()
        .map(|a| a.to_admin_list_item(&state.hashid_manager))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}

/// Search articles handler for admin.
///
/// Returns a list of articles matching the search query.
/// Search is performed on title and content fields.
///
/// # Arguments
/// * `state` - Application state containing database pool and HashID manager
/// * `query` - Search query parameters including search keyword and page size
///
/// # Returns
/// Returns a list of articles matching the search query.
///
/// # Errors
/// Returns `Database` error on database failures.
//
// // 管理员搜索文章处理器。
// //
// // 返回匹配搜索查询的文章列表。
// // 搜索在标题和内容字段上执行。
// //
// // # 参数
// // * `state` - 包含数据库连接池和 HashID 管理器的应用状态
// // * `query` - 搜索查询参数，包括搜索关键字和页面大小
// //
// // # 返回
// // 返回匹配搜索查询的文章列表。
// //
// // # 错误
// // 数据库失败时返回 `Database` 错误。
pub async fn search_articles(
    State(state): State<AdminState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page_size = query.page_size.unwrap_or(20).max(1);
    let search_term = query.q.unwrap_or_default();
    let search_pattern = format!("%{}%", search_term);

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
         FROM articles
         WHERE title LIKE ? OR content LIKE ?
         ORDER BY publish_at DESC
         LIMIT ?"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(page_size)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let responses: Vec<AdminArticleListItem> = articles
        .iter()
        .map(|a| a.to_admin_list_item(&state.hashid_manager))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}

/// Get total pages for search results for admin.
///
/// Returns the total number of pages and items based on page_size for search results.
//
// // 获取管理员搜索结果总页数。
// //
// // 基于 page_size 返回搜索结果的总页数和总项目数。
pub async fn get_search_page_count(
    State(state): State<AdminState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page_size = query.page_size.unwrap_or(20).max(1);
    let search_term = query.q.unwrap_or_default();
    let search_pattern = format!("%{}%", search_term);

    let total_items: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM articles WHERE title LIKE ? OR content LIKE ?"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let total_items = total_items.0 as u32;
    let total_pages = (total_items + page_size - 1) / page_size;

    Ok(Json(ApiResponse {
        success: true,
        data: PageCountResponse {
            total_pages,
            total_items,
        },
    }))
}

/// Search articles paginated for admin.
///
/// Returns a specific page of articles matching the search query based on page and page_size.
//
// // 管理员分页搜索文章。
// //
// // 基于 page 和 page_size 返回匹配搜索查询的特定页文章。
pub async fn search_articles_paginated(
    State(state): State<AdminState>,
    Path(page): Path<u32>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page_size = query.page_size.unwrap_or(20).max(1);
    let page = page.max(1);
    let offset = (page - 1) * page_size;
    let search_term = query.q.unwrap_or_default();
    let search_pattern = format!("%{}%", search_term);

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
         FROM articles
         WHERE title LIKE ? OR content LIKE ?
         ORDER BY publish_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let responses: Vec<AdminArticleListItem> = articles
        .iter()
        .map(|a| a.to_admin_list_item(&state.hashid_manager))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse {
        success: true,
        data: responses,
    }))
}
