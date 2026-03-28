use serde::{Deserialize, Serialize};

/// User subscription model.
//
// // 用户订阅模型。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub id: i64,
    pub user_id: i64,
    pub tier: i32,
    pub start_date: i64,
    pub end_date: i64,
    pub created_at: i64,
}
