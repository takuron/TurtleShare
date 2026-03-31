// Admin user management handlers
//
// // 管理员用户管理处理器

use super::auth::AdminState;
use crate::error::AppError;
use crate::handlers::common::ApiResponse;
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User, UserResponse};
use crate::utils::hash;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// List all users.
///
/// Returns a list of all users.
//
// // 列出所有用户。
// //
// // 返回所有用户的列表。
pub async fn list_users(State(state): State<AdminState>) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 转换为带有 hash_id 的响应
    let user_responses: Vec<UserResponse> = users
        .iter()
        .map(|u| u.to_response(state.hashid_manager.encode(u.id).unwrap_or_default()))
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: user_responses,
    }))
}

/// Get user detail.
///
/// Retrieves a single user by hash_id.
//
// // 获取用户详情。
// //
// // 通过 hash_id 检索单个用户。
pub async fn get_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: user.to_response(hash_id),
    }))
}

/// Create user.
///
/// Creates a new user with hashed password.
//
// // 创建用户。
// //
// // 创建带有哈希密码的新用户。
pub async fn create_user(
    State(state): State<AdminState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let password_hash = hash::hash_password(&req.password)?;
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let id = sqlx::query(
        "INSERT INTO users (username, password_hash, email, note, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(&req.email)
    .bind(&req.note)
    .bind(created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            AppError::ValidationError("Username already exists".to_string())
        } else {
            AppError::Database(e.to_string())
        }
    })?
    .last_insert_rowid();

    let user = User {
        id,
        username: req.username,
        password_hash,
        email: req.email,
        note: req.note,
        created_at,
    };

    let hash_id = state.hashid_manager.encode(id)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: user.to_response(hash_id),
        }),
    ))
}

/// Update user.
///
/// Updates an existing user's information.
//
// // 更新用户。
// //
// // 更新现有用户的信息。
pub async fn update_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let mut user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email, note, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if let Some(username) = req.username {
        user.username = username;
    }
    if let Some(password) = req.password {
        user.password_hash = hash::hash_password(&password)?;
    }
    if let Some(email) = req.email {
        if email.is_empty() {
            user.email = None;
        } else {
            user.email = Some(email);
        }
    }
    if let Some(note) = req.note {
        if note.is_empty() {
            user.note = None;
        } else {
            user.note = Some(note);
        }
    }

    sqlx::query(
        "UPDATE users SET username = ?, password_hash = ?, email = ?, note = ? WHERE id = ?",
    )
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.email)
    .bind(&user.note)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: user.to_response(hash_id),
    }))
}

/// Delete user.
///
/// Removes a user from the database.
//
// // 删除用户。
// //
// // 从数据库中移除用户。
pub async fn delete_user(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let rows_affected = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(ApiResponse {
        success: true,
        data: serde_json::json!({ "deleted": true }),
    }))
}

/// Query parameters for get_user_tier.
//
// // get_user_tier 的查询参数。
#[derive(Deserialize)]
pub struct TierQuery {
    pub at: Option<i64>,
}

/// Response for get_user_tier.
//
// // get_user_tier 的响应。
#[derive(Serialize)]
pub struct TierResponse {
    pub tier: i64,
}

/// Get user tier.
///
/// Queries a user's subscription tier at a specific time.
//
// // 获取用户等级。
// //
// // 查询用户在特定时间的订阅等级。
pub async fn get_user_tier(
    State(state): State<AdminState>,
    Path(hash_id): Path<String>,
    Query(query): Query<TierQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 解码 hash_id 为数字 ID
    let id = state.hashid_manager.decode(&hash_id)?;

    let at = query.at.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT MAX(tier) FROM user_subscriptions WHERE user_id = ? AND start_date <= ? AND end_date >= ?"
    )
    .bind(id)
    .bind(at)
    .bind(at)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let tier = result.map(|(t,)| t).unwrap_or(0);

    Ok(Json(ApiResponse {
        success: true,
        data: TierResponse { tier },
    }))
}
