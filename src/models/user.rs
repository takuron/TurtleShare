use serde::{Deserialize, Serialize};

/// User model representing a registered user.
//
// // 用户模型，表示已注册用户。
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
}

/// Request payload for user registration.
//
// // 用户注册请求载荷。
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub note: Option<String>,
}

/// Request payload for user login.
//
// // 用户登录请求载荷。
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Request payload for updating a user.
//
// // 更新用户请求载荷。
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}
