use crate::error::{AppError, Result};
use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// JWT claims structure.
///
/// Contains user identity and role information.
/// For admin: sub = "admin"
/// For users: sub = "user:<user_hashid>"
//
// // JWT 声明结构。
// //
// // 包含用户身份和角色信息。
// // 管理员：sub = "admin"
// // 用户：sub = "user:<用户HashID>"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

/// JWT manager with key rotation support.
///
/// Handles token generation, verification, and automatic key rotation.
//
// // 支持密钥轮换的 JWT 管理器。
// //
// // 处理令牌生成、验证和自动密钥轮换。
pub struct JwtManager {
    pool: SqlitePool,
    config_secret: String,
    expiry_hours: u64,
    rotation_days: u64,
}

impl JwtManager {
    /// Creates a new JWT manager and initializes secrets if needed.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `base_secret` - Base secret from config.toml
    /// * `expiry_hours` - Token expiry time in hours
    /// * `rotation_days` - Days before rotating secrets
    //
    // // 创建新的 JWT 管理器并在需要时初始化密钥。
    // //
    // // # 参数
    // // * `pool` - 数据库连接池
    // // * `base_secret` - 来自 config.toml 的基础密钥
    // // * `expiry_hours` - 令牌过期时间（小时）
    // // * `rotation_days` - 密钥轮换前的天数
    pub async fn new(
        pool: SqlitePool,
        base_secret: String,
        expiry_hours: u64,
        rotation_days: u64,
    ) -> Result<Self> {
        let manager = Self {
            pool,
            config_secret: base_secret,
            expiry_hours,
            rotation_days,
        };

        // 1. 初始化或轮换密钥
        manager.initialize_or_rotate().await?;

        Ok(manager)
    }

    /// Generates a new secret using SHA256(config_secret + UUID).
    //
    // // 使用 SHA256(config_secret + UUID) 生成新密钥。
    fn generate_secret(&self) -> String {
        let uuid = Uuid::new_v4().to_string();
        let input = format!("{}{}", self.config_secret, uuid);
        let hash = Sha256::digest(input.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(hash)
    }

    /// Initializes secrets on first run or rotates if needed.
    //
    // // 首次运行时初始化密钥或在需要时轮换。
    async fn initialize_or_rotate(&self) -> Result<()> {
        // 1. 检查当前密钥是否存在
        let current: Option<(String,)> =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_current'")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        if current.is_none() {
            // 2. 首次初始化
            self.initialize_secrets().await?;
        } else {
            // 3. 检查是否需要轮换
            self.check_and_rotate().await?;
        }

        Ok(())
    }

    /// Initializes secrets for the first time.
    //
    // // 首次初始化密钥。
    async fn initialize_secrets(&self) -> Result<()> {
        let secret = self.generate_secret();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 1. 插入当前密钥
        sqlx::query(
            "INSERT INTO kv_store (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind("jwt_secret_current")
        .bind(&secret)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. 插入创建时间
        sqlx::query(
            "INSERT INTO kv_store (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind("jwt_secret_date")
        .bind(now.to_string())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Checks if rotation is needed and performs it.
    //
    // // 检查是否需要轮换并执行。
    pub async fn check_and_rotate(&self) -> Result<()> {
        // 1. 获取密钥创建时间
        let date_str: (String,) =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_date'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        let secret_timestamp: i64 = date_str
            .0
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid timestamp format: {}", e)))?;

        // 2. 检查是否超过轮换周期
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let days_elapsed = (now - secret_timestamp) / 86400;

        if days_elapsed >= self.rotation_days as i64 {
            self.rotate_secrets().await?;
        }

        Ok(())
    }

    /// Rotates secrets by moving current to previous and generating new current.
    //
    // // 通过将当前密钥移至上一个并生成新的当前密钥来轮换密钥。
    async fn rotate_secrets(&self) -> Result<()> {
        // 1. 获取当前密钥
        let current: (String,) =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_current'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. 生成新密钥
        let new_secret = self.generate_secret();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 3. 更新或插入上一个密钥
        sqlx::query(
            "INSERT INTO kv_store (key, value, created_at, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?",
        )
        .bind("jwt_secret_previous")
        .bind(&current.0)
        .bind(now)
        .bind(now)
        .bind(&current.0)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 4. 更新当前密钥
        sqlx::query(
            "UPDATE kv_store SET value = ?, updated_at = ? WHERE key = 'jwt_secret_current'",
        )
        .bind(&new_secret)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 5. 更新密钥创建时间
        sqlx::query("UPDATE kv_store SET value = ?, updated_at = ? WHERE key = 'jwt_secret_date'")
            .bind(now.to_string())
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Generates a JWT token.
    ///
    /// # Arguments
    /// * `sub` - Subject ("admin" or "user:<user_hashid>")
    /// * `name` - Display name for the token
    /// * `role` - User role ("admin" or "user")
    //
    // // 生成 JWT 令牌。
    // //
    // // # 参数
    // // * `sub` - 主题（"admin" 或 "user:<用户HashID>"）
    // // * `name` - 令牌的显示名称
    // // * `role` - 用户角色（"admin" 或 "user"）
    pub async fn generate_token(&self, sub: &str, name: &str, role: &str) -> Result<String> {
        // 1. 获取当前密钥
        let secret: (String,) =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_current'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. 创建声明
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let exp = now + (self.expiry_hours as i64 * 3600);

        let claims = Claims {
            sub: sub.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            exp,
            iat: now,
        };

        // 3. 编码令牌
        let secret_bytes = base64::engine::general_purpose::STANDARD
            .decode(&secret.0)
            .map_err(|e| AppError::Internal(format!("Invalid secret format: {}", e)))?;

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret_bytes),
        )
        .map_err(|e| AppError::Internal(format!("Failed to encode token: {}", e)))?;

        Ok(token)
    }

    /// Verifies a JWT token using current or previous secret.
    ///
    /// # Arguments
    /// * `token` - The JWT token to verify
    ///
    /// # Returns
    /// Returns the decoded claims if valid.
    //
    // // 使用当前或上一个密钥验证 JWT 令牌。
    // //
    // // # 参数
    // // * `token` - 要验证的 JWT 令牌
    // //
    // // # 返回
    // // 如果有效，返回解码的声明。
    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        // 1. 获取当前密钥
        let current: (String,) =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_current'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. 尝试使用当前密钥验证
        if let Ok(claims) = self.verify_with_secret(token, &current.0) {
            return Ok(claims);
        }

        // 3. 尝试使用上一个密钥验证
        let previous: Option<(String,)> =
            sqlx::query_as("SELECT value FROM kv_store WHERE key = 'jwt_secret_previous'")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(prev) = previous {
            if let Ok(claims) = self.verify_with_secret(token, &prev.0) {
                return Ok(claims);
            }
        }

        Err(AppError::Unauthorized("Invalid token".to_string()))
    }

    /// Verifies token with a specific secret.
    //
    // // 使用特定密钥验证令牌。
    fn verify_with_secret(&self, token: &str, secret: &str) -> Result<Claims> {
        let secret_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret)
            .map_err(|e| AppError::Internal(format!("Invalid secret format: {}", e)))?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data =
            decode::<Claims>(token, &DecodingKey::from_secret(&secret_bytes), &validation)
                .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }
}
