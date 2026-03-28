use serde::{Deserialize, Serialize};

/// User model representing a registered user.
//
// // 用户模型，表示已注册用户。
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
}

/// User response with hash_id for API responses.
//
// // 带有 hash_id 的用户响应，用于 API 响应。
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub hash_id: String,
    pub username: String,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
}

impl User {
    /// Converts User to UserResponse with encoded hash_id.
    //
    // // 将 User 转换为带有编码 hash_id 的 UserResponse。
    pub fn to_response(&self, hash_id: String) -> UserResponse {
        UserResponse {
            hash_id,
            username: self.username.clone(),
            email: self.email.clone(),
            note: self.note.clone(),
            created_at: self.created_at,
        }
    }
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
