// Tier description model - Stored as JSON in kv_store
//
// // 等级说明模型 - 以 JSON 格式存储在 kv_store 中

use serde::{Deserialize, Serialize};

/// A single tier description entry.
///
/// Represents the description and pricing information for a subscription tier.
/// Each tier is identified by its numeric tier level.
//
// // 单个等级说明条目。
// //
// // 表示订阅等级的说明和价格信息。
// // 每个等级通过其数字等级标识。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDescription {
    /// The subscription tier level (0-255).
    //
    // // 订阅等级（0-255）。
    pub tier: u8,

    /// The display name of the tier.
    //
    // // 等级的显示名称。
    pub name: String,

    /// The plain-text description of the tier.
    //
    // // 等级的纯文本说明。
    pub description: String,

    /// The plain-text price information for the tier.
    //
    // // 等级的纯文本价格信息。
    pub price: String,

    /// The purchase link URL for the tier.
    //
    // // 等级的购买链接 URL。
    pub purchase_url: String,
}

/// All tier descriptions stored in kv_store under the key "tier_descriptions".
///
/// Contains a list of tier description entries and the timestamp of the last update.
/// The entire struct is serialized as a JSON string and stored in the `value` column.
//
// // 存储在 kv_store 中键为 "tier_descriptions" 的所有等级说明。
// //
// // 包含等级说明条目列表和最后更新时间戳。
// // 整个结构体序列化为 JSON 字符串并存储在 `value` 列中。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDescriptionsData {
    /// List of tier description entries, sorted by tier level.
    //
    // // 等级说明条目列表，按等级排序。
    pub tiers: Vec<TierDescription>,

    /// Unix timestamp of the last update.
    //
    // // 最后更新的 Unix 时间戳。
    pub updated_at: i64,
}

/// Request body for upserting a tier description.
///
/// The `name`, `description`, `price`, and `purchase_url` fields are all optional individually,
/// but at least one of them must be non-empty when making a request.
/// On update, only provided non-empty fields are overwritten.
//
// // 添加或更新等级说明的请求体。
// //
// // `name`、`description`、`price` 和 `purchase_url` 字段均可单独省略，
// // 但请求中至少有一个字段必须非空。
// // 更新时，仅覆盖提供的非空字段。
#[derive(Debug, Deserialize)]
pub struct UpsertTierDescriptionRequest {
    /// The subscription tier level (0-255).
    //
    // // 订阅等级（0-255）。
    pub tier: u8,

    /// The display name of the tier. Optional on update.
    //
    // // 等级的显示名称。更新时可选。
    pub name: Option<String>,

    /// The plain-text description of the tier. Optional on update.
    //
    // // 等级的纯文本说明。更新时可选。
    pub description: Option<String>,

    /// The plain-text price information for the tier. Optional on update.
    //
    // // 等级的纯文本价格信息。更新时可选。
    pub price: Option<String>,

    /// The purchase link URL for the tier. Optional on update.
    //
    // // 等级的购买链接 URL。更新时可选。
    pub purchase_url: Option<String>,
}
