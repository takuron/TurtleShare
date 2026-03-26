use serde::{Deserialize, Serialize};

/// User subscription model.
//
// // 用户订阅模型。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub id: i64,
    pub user_id: i64,
    pub tier: i32,
    pub start_date: String,
    pub end_date: String,
    pub created_at: String,
}
