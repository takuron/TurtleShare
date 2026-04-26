use crate::error::{AppError, Result};
use harsh::Harsh;

/// HashID manager for encoding/decoding user IDs.
///
/// Provides a way to hide auto-increment IDs by encoding them as short hash strings.
//
// // HashID 管理器，用于编码/解码用户 ID。
// //
// // 提供一种通过将自增 ID 编码为短哈希字符串来隐藏它们的方法。
pub struct HashIdManager {
    harsh: Harsh,
}

impl HashIdManager {
    /// Creates a new HashID manager with base secret from config.
    ///
    /// # Arguments
    /// * `base_secret` - Base secret from config (same as JWT base_secret)
    /// * `min_length` - Minimum length of encoded IDs
    //
    // // 使用配置中的基础密钥创建新的 HashID 管理器。
    // //
    // // # 参数
    // // * `base_secret` - 来自配置的基础密钥（与 JWT base_secret 相同）
    // // * `min_length` - 编码 ID 的最小长度
    pub fn new(base_secret: &str, min_length: usize) -> Result<Self> {
        let harsh = Harsh::builder()
            .salt(base_secret)
            .length(min_length)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HashID: {}", e)))?;

        Ok(Self { harsh })
    }

    /// Encodes a user ID to a hash string.
    ///
    /// # Arguments
    /// * `id` - The numeric user ID
    ///
    /// # Returns
    /// Returns the encoded hash string.
    //
    // // 将用户 ID 编码为哈希字符串。
    // //
    // // # 参数
    // // * `id` - 数字用户 ID
    // //
    // // # 返回
    // // 返回编码的哈希字符串。
    pub fn encode(&self, id: i64) -> Result<String> {
        if id < 0 {
            return Err(AppError::Internal("ID must be non-negative".to_string()));
        }

        Ok(self.harsh.encode(&[id as u64]))
    }

    /// Decodes a hash string back to a user ID.
    ///
    /// # Arguments
    /// * `hash` - The encoded hash string
    ///
    /// # Returns
    /// Returns the decoded numeric ID.
    //
    // // 将哈希字符串解码回用户 ID。
    // //
    // // # 参数
    // // * `hash` - 编码的哈希字符串
    // //
    // // # 返回
    // // 返回解码的数字 ID。
    pub fn decode(&self, hash: &str) -> Result<i64> {
        let decoded = self
            .harsh
            .decode(hash)
            .map_err(|_| AppError::InvalidHashId(format!("Invalid hash ID: {}", hash)))?;

        if decoded.is_empty() {
            return Err(AppError::InvalidHashId(format!(
                "Invalid hash ID: {}",
                hash
            )));
        }

        Ok(decoded[0] as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let manager = HashIdManager::new("test_salt_123", 6).unwrap();

        let id = 12345;
        let hash = manager.encode(id).unwrap();
        let decoded = manager.decode(&hash).unwrap();

        assert_eq!(id, decoded);
        assert!(hash.len() >= 6);
    }

    #[test]
    fn test_negative_id() {
        let manager = HashIdManager::new("test_salt_123", 6).unwrap();
        assert!(manager.encode(-1).is_err());
    }

    #[test]
    fn test_invalid_hash() {
        let manager = HashIdManager::new("test_salt_123", 6).unwrap();
        assert!(manager.decode("invalid!!!").is_err());
    }
}
