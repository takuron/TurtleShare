use crate::error::AppError;
use crate::utils::hashid::HashIdManager;
use serde::{Deserialize, Serialize};

/// User subscription model representing a subscription record in the database.
///
/// Maps directly to the `user_subscriptions` table.
//
// // 用户订阅模型，表示数据库中的订阅记录。
// //
// // 直接映射到 `user_subscriptions` 表。
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserSubscription {
    pub id: i64,
    pub user_id: i64,
    pub tier: i32,
    pub start_date: i64,
    pub end_date: i64,
    /// Admin-only note for this subscription period.
    /// / 仅管理员可见的订阅时段备注。
    pub note: Option<String>,
    pub created_at: i64,
}

/// Subscription response with user hash_id for API responses.
///
/// Exposes the subscription data with user_hash_id instead of the raw user_id.
/// The note field is included as it's only accessible via admin endpoints.
//
// // 带有用户 hash_id 的订阅响应，用于 API 响应。
// //
// // 暴露订阅数据时使用 user_hash_id 代替原始 user_id。
// // note 字段包含在内，因为它只能通过管理员端点访问。
#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub hash_id: String,
    pub user_hash_id: String,
    pub tier: i32,
    pub start_date: i64,
    pub end_date: i64,
    /// Admin-only note for this subscription period.
    /// / 仅管理员可见的订阅时段备注。
    pub note: Option<String>,
    pub created_at: i64,
}

impl UserSubscription {
    /// Converts UserSubscription to SubscriptionResponse with encoded hash IDs.
    ///
    /// # Arguments
    /// * `hash_id_manager` - The HashID manager for encoding IDs
    /// * `user_hash_id` - The encoded hash ID of the user
    ///
    /// # Returns
    /// Returns the SubscriptionResponse with encoded hash IDs.
    ///
    /// # Errors
    /// Returns `AppError` if ID encoding fails.
    //
    // // 将 UserSubscription 转换为带有编码哈希 ID 的 SubscriptionResponse。
    // //
    // // # 参数
    // // * `hash_id_manager` - 用于编码 ID 的 HashID 管理器
    // // * `user_hash_id` - 用户的编码哈希 ID
    // //
    // // # 返回
    // // 返回带有编码哈希 ID 的 SubscriptionResponse。
    // //
    // // # 错误
    // // 如果 ID 编码失败，返回 `AppError`。
    pub fn to_response(
        &self,
        hash_id_manager: &HashIdManager,
        user_hash_id: String,
    ) -> Result<SubscriptionResponse, AppError> {
        Ok(SubscriptionResponse {
            hash_id: hash_id_manager.encode(self.id)?,
            user_hash_id,
            tier: self.tier,
            start_date: self.start_date,
            end_date: self.end_date,
            note: self.note.clone(),
            created_at: self.created_at,
        })
    }
}

/// Request payload for creating a new subscription.
///
/// Used when adding a subscription period to a user.
//
// // 创建新订阅的请求载荷。
// //
// // 用于向用户添加订阅时段。
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    /// The subscription tier level (e.g., 0, 1, 2, 3)
    /// / 订阅等级（例如 0, 1, 2, 3）
    pub tier: i32,

    /// Start date as Unix timestamp (seconds since epoch)
    /// / 开始日期，Unix 时间戳（自纪元以来的秒数）
    pub start_date: i64,

    /// End date as Unix timestamp (seconds since epoch)
    /// / 结束日期，Unix 时间戳（自纪元以来的秒数）
    pub end_date: i64,

    /// Optional admin-only note for this subscription
    /// / 可选的仅管理员可见的订阅备注
    pub note: Option<String>,
}

/// Request payload for updating an existing subscription.
///
/// All fields are optional; only provided fields will be updated.
//
// // 更新现有订阅的请求载荷。
// //
// // 所有字段都是可选的；仅更新提供的字段。
#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    /// Optional new tier level
    /// / 可选的新等级
    pub tier: Option<i32>,

    /// Optional new start date as Unix timestamp
    /// / 可选的新开始日期，Unix 时间戳
    pub start_date: Option<i64>,

    /// Optional new end date as Unix timestamp
    /// / 可选的新结束日期，Unix 时间戳
    pub end_date: Option<i64>,

    /// Optional admin-only note for this subscription
    /// / 可选的仅管理员可见的订阅备注
    pub note: Option<String>,
}
