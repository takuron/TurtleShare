// User articles handler with time-based tier access control
//
// // 用户文章处理器，带基于时间的等级访问控制

use super::auth::UserState;
use crate::error::AppError;
use crate::handlers::common::{ApiResponse, PageCountResponse, PaginationQuery};
use crate::middleware::auth::AuthClaims;
use crate::models::article::Article;
use crate::utils::hashid::HashIdManager;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Serialize;

/// Calculates the user's subscription tier at a specific time.
///
/// Returns the maximum tier from all active subscriptions at the given timestamp.
/// Returns 0 if no active subscriptions exist at that time.
//
// // 计算用户在特定时间的订阅等级。
// //
// // 返回在给定时间戳所有活跃订阅中的最高等级。
// // 如果该时间没有活跃订阅，返回 0。
async fn get_user_tier_at_time(
    pool: &sqlx::SqlitePool,
    user_id: i64,
    at_time: i64,
) -> Result<i32, AppError> {
    // 查询用户在指定时间的最高订阅等级
    let result: Option<(i32,)> = sqlx::query_as(
        "SELECT MAX(tier) FROM user_subscriptions WHERE user_id = ? AND start_date <= ? AND end_date >= ?"
    )
    .bind(user_id)
    .bind(at_time)
    .bind(at_time)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 如果没有活跃订阅，默认等级为 0
    Ok(result.map(|(t,)| t).unwrap_or(0))
}

/// Article list item for user-facing article list.
///
/// Does not include content, is_public, and file_links fields.
/// Includes accessible field to indicate if user can fully access the article.
//
// // 用户面向的文章列表项。
// //
// // 不包含 content、is_public 和 file_links 字段。
// // 包含 accessible 字段以指示用户是否可以完整访问文章。
#[derive(Debug, Serialize, Clone)]
pub struct ArticleListItem {
    pub hash_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub required_tier: i32,
    /// Whether user can fully access the article content.
    /// / 用户是否可以完整访问文章内容。
    pub accessible: bool,
    pub publish_at: i64,
    pub updated_at: i64,
}

impl Article {
    /// Converts Article to ArticleListItem with accessibility check.
    ///
    /// # Arguments
    /// * `hash_id_manager` - The HashID manager for encoding IDs
    /// * `user_tier_at_publish` - User's tier at the article's publish time
    ///
    /// # Returns
    /// Returns the ArticleListItem with accessibility determined by tier comparison.
    //
    // // 将 Article 转换为带有可访问性检查的 ArticleListItem。
    // //
    // // # 参数
    // // * `hash_id_manager` - 用于编码 ID 的 HashID 管理器
    // // * `user_tier_at_publish` - 用户在文章发布时间的等级
    // //
    // // # 返回
    // // 返回通过等级比较确定可访问性的 ArticleListItem。
    fn to_list_item(
        &self,
        hash_id_manager: &HashIdManager,
        user_tier_at_publish: i32,
    ) -> Result<ArticleListItem, AppError> {
        // 判断用户在文章发布时是否有足够等级完整访问
        let accessible = user_tier_at_publish >= self.required_tier;

        Ok(ArticleListItem {
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

/// List articles handler for users.
///
/// Returns a list of articles visible to the authenticated user.
/// An article is visible if:
/// - User had sufficient tier at article publish time (accessible = true), OR
/// - Article is public (is_public = true) but user didn't have tier (accessible = false)
///
/// The content and is_public fields are excluded from the response.
/// An accessible field indicates if the user can fully access the article.
///
/// # Arguments
/// * `state` - Application state containing database pool and HashID manager
/// * `claims` - Authenticated user claims from JWT
///
/// # Returns
/// Returns a list of visible articles with accessibility indicators.
///
/// # Errors
/// Returns `Internal` error if token subject format is invalid.
/// Returns `Database` error on database failures.
//
// // 用户文章列表处理器。
// //
// // 返回已认证用户可见的文章列表。
// // 文章可见的条件：
// // - 用户在文章发布时有足够等级（accessible = true），或者
// // - 文章是公开的（is_public = true）但用户没有足够等级（accessible = false）
// //
// // 响应中不包含 content 和 is_public 字段。
// // accessible 字段指示用户是否可以完整访问文章。
// //
// // # 参数
// // * `state` - 包含数据库连接池和 HashID 管理器的应用状态
// // * `claims` - 来自 JWT 的已认证用户声明
// //
// // # 返回
// // 返回带有可访问性指示的可见文章列表。
// //
// // # 错误
// // 如果令牌主题格式无效，返回 `Internal` 错误。
// // 数据库失败时返回 `Database` 错误。
pub async fn list_articles(
    State(state): State<UserState>,
    claims: AuthClaims,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 JWT sub 字段提取用户 hash_id（格式为 "user:<hash_id>"）
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    // 2. 解码 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(user_hash_id)?;

    // 3. 查询所有文章（公开文章和需要等级的文章都查询）
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
         FROM articles
         ORDER BY publish_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. 过滤并转换文章
    let mut items: Vec<ArticleListItem> = Vec::new();
    for article in &articles {
        // 获取用户在文章发布时间的等级
        let user_tier_at_publish =
            get_user_tier_at_time(&state.pool, user_id, article.publish_at).await?;

        // 判断是否可以完整访问
        let can_access = user_tier_at_publish >= article.required_tier;

        // 如果可以完整访问，或者文章是公开的，则显示在列表中
        if can_access || article.is_public {
            items.push(article.to_list_item(&state.hashid_manager, user_tier_at_publish)?);
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: items,
    }))
}

/// Get article detail handler with time-based access control.
///
/// Retrieves a single article by hash_id if the user had sufficient tier
/// at the article's publish time.
///
/// Note: Public status (is_public) does NOT grant access to full article content.
/// Access is solely determined by the user's subscription tier at publish time.
///
/// # Arguments
/// * `state` - Application state containing database pool and HashID manager
/// * `claims` - Authenticated user claims from JWT
/// * `hash_id` - The hash ID of the article to retrieve
///
/// # Returns
/// Returns the full article details if access is granted.
///
/// # Errors
/// Returns `Internal` error if token subject format is invalid.
/// Returns `NotFound` if article does not exist.
/// Returns `Forbidden` if user did not have sufficient tier at article publish time.
/// Returns `Database` error on database failures.
//
// // 获取文章详情处理器，带基于时间的访问控制。
// //
// // 如果用户在文章发布时有足够等级，通过 hash_id 检索单篇文章。
// //
// // 注意：公开状态（is_public）不授予完整文章内容的访问权限。
// // 访问权限仅由用户在发布时间的订阅等级决定。
// //
// // # 参数
// // * `state` - 包含数据库连接池和 HashID 管理器的应用状态
// // * `claims` - 来自 JWT 的已认证用户声明
// // * `hash_id` - 要检索的文章的 hash ID
// //
// // # 返回
// // 如果授予访问权限，返回完整的文章详情。
// //
// // # 错误
// // 如果令牌主题格式无效，返回 `Internal` 错误。
// // 如果文章不存在，返回 `NotFound`。
// // 如果用户在文章发布时没有足够等级，返回 `Forbidden`。
// // 数据库失败时返回 `Database` 错误。
pub async fn get_article(
    State(state): State<UserState>,
    claims: AuthClaims,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 从 JWT sub 字段提取用户 hash_id（格式为 "user:<hash_id>"）
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    // 2. 解码用户 hash_id 为数字 ID
    let user_id = state.hashid_manager.decode(user_hash_id)?;

    // 3. 解码文章 hash_id 为数字 ID
    let article_id = state.hashid_manager.decode(&hash_id)?;

    // 4. 查询文章
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at FROM articles WHERE id = ?"
    )
    .bind(article_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    // 5. 获取用户在文章发布时间的等级
    let user_tier_at_publish =
        get_user_tier_at_time(&state.pool, user_id, article.publish_at).await?;

    // 6. 检查访问权限：仅基于发布时间的等级，不考虑 is_public
    if user_tier_at_publish < article.required_tier {
        return Err(AppError::Forbidden(
            "Insufficient subscription tier to access this article".to_string(),
        ));
    }

    // 7. 返回文章完整详情
    Ok(Json(ApiResponse {
        success: true,
        data: article.to_client_detail_response(&state.hashid_manager)?,
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
    State(state): State<UserState>,
    claims: AuthClaims,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    let user_id = state.hashid_manager.decode(user_hash_id)?;
    let page_size = query.page_size.unwrap_or(20).max(1);

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
         FROM articles"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut total_items = 0;
    for article in &articles {
        let user_tier_at_publish =
            get_user_tier_at_time(&state.pool, user_id, article.created_at).await?;

        let can_access = user_tier_at_publish >= article.required_tier;

        if can_access || article.is_public {
            total_items += 1;
        }
    }

    let total_pages = (total_items + page_size - 1) / page_size;

    Ok(Json(ApiResponse {
        success: true,
        data: PageCountResponse {
            total_pages,
            total_items,
        },
    }))
}

/// List articles paginated for users.
///
/// Returns a specific page of visible articles based on page and page_size.
//
// // 分页列出用户可见文章。
// //
// // 基于 page 和 page_size 返回特定页的可见文章。
pub async fn list_articles_paginated(
    State(state): State<UserState>,
    claims: AuthClaims,
    Path(page): Path<u32>,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_hash_id = claims
        .0
        .sub
        .strip_prefix("user:")
        .ok_or_else(|| AppError::Internal("Invalid token subject format".to_string()))?;

    let user_id = state.hashid_manager.decode(user_hash_id)?;

    let page_size = query.page_size.unwrap_or(20).max(1);
    let page = page.max(1);
    let offset = (page - 1) * page_size;

    // For user paginated list, we might need a more complex query if we want strict DB pagination
    // But since the existing implementation filters in memory based on user tier at publish time,
    // and 'is_public', we must either do the logic in SQL or stick to memory filter.
    // If we filter in memory, LIMIT/OFFSET in SQL isn't accurate for the final list size.
    // However, given the current scope, we will fetch all, filter, and then slice. This preserves existing behavior, though less efficient.
    // A better approach in the future would be a JOIN with user_subscriptions, but that changes the architecture.
    // To implement `page` correctly with the memory filter:

    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, cover_image, content, required_tier, is_public, file_links, publish_at, created_at, updated_at
         FROM articles
         ORDER BY publish_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut items: Vec<ArticleListItem> = Vec::new();
    for article in &articles {
        let user_tier_at_publish =
            get_user_tier_at_time(&state.pool, user_id, article.created_at).await?;

        let can_access = user_tier_at_publish >= article.required_tier;

        if can_access || article.is_public {
            items.push(article.to_list_item(&state.hashid_manager, user_tier_at_publish)?);
        }
    }

    let total_filtered = items.len();

    // Manual pagination
    let start_index = offset as usize;
    let end_index = std::cmp::min(start_index + page_size as usize, total_filtered);

    let paginated_items = if start_index < total_filtered {
        items[start_index..end_index].to_vec()
    } else {
        vec![]
    };

    Ok(Json(ApiResponse {
        success: true,
        data: paginated_items,
    }))
}
