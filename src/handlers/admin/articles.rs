// Admin article management handlers
//
// // 管理员文章管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::article::{
    Article, ArticleResponse, CreateArticleRequest, UpdateArticleRequest, serialize_file_links,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// List all articles.
///
/// Returns a list of all articles ordered by created_at descending.
//
// // 列出所有文章。
// //
// // 返回按 created_at 降序排列的所有文章列表。
pub async fn list_articles(
    State(state): State<AdminState>,
) -> Result<impl IntoResponse, AppError> {
    // 查询所有文章，按创建时间降序排列
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at FROM articles ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 转换为带有 hash_id 的响应
    let responses: Vec<ArticleResponse> = articles
        .iter()
        .map(|a| a.to_response(&state.hashid_manager))
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
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at FROM articles WHERE id = ?"
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
    // 1. 验证 required_tier 非负
    if req.required_tier < 0 {
        return Err(AppError::ValidationError(
            "required_tier must be non-negative".to_string(),
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

    // 4. 插入文章记录
    let id = sqlx::query(
        "INSERT INTO articles (title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.title)
    .bind(&req.cover_image)
    .bind(&req.content)
    .bind(req.required_tier)
    .bind(req.is_public)
    .bind(&file_links_json)
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
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, created_at, updated_at FROM articles WHERE id = ?"
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
        if required_tier < 0 {
            return Err(AppError::ValidationError(
                "required_tier must be non-negative".to_string(),
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

    // 4. 更新 updated_at 时间戳
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    article.updated_at = now;

    // 5. 更新数据库
    sqlx::query(
        "UPDATE articles SET title = ?, cover_image = ?, content = ?, required_tier = ?, is_public = ?, file_links = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&article.title)
    .bind(&article.cover_image)
    .bind(&article.content)
    .bind(article.required_tier)
    .bind(article.is_public)
    .bind(&article.file_links)
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
