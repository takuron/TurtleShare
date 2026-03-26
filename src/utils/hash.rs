use argon2::{Argon2, Algorithm, Version, Params, PasswordHasher};
use password_hash::phc::PasswordHash;
use password_hash::PasswordVerifier;
use crate::error::{AppError, Result};

/// Creates Argon2id parameters: t=2, m=19456 (19MB), p=1
//
// // 创建 Argon2id 参数：t=2, m=19456 (19MB), p=1
fn get_argon2_params() -> Params {
    Params::new(19456, 2, 1, Some(32)).expect("Invalid Argon2 parameters")
}

/// Hashes a password with standard Argon2id.
///
/// Parameters used:
/// - t_cost (Iterations): 2
/// - m_cost (Memory): 19456 (19MB)
/// - p_cost (Parallelism): 1
/// - Output Hash Length: 32 bytes
///
/// # Arguments
/// * `password` - The raw password to hash.
///
/// # Returns
/// Returns a string representing the PHC formatted hash.
//
// // 使用标准 Argon2id 对密码进行哈希。
// //
// // 使用的参数：
// // - t_cost (循环次数): 2
// // - m_cost (内存开销): 19456 (19MB)
// // - p_cost (并行线程): 1
// // - 输出哈希长度: 32 字节
// //
// // # 参数
// // * `password` - 要哈希的原始密码。
// //
// // # 返回
// // 返回 PHC 格式的哈希字符串。
pub fn hash_password(password: &str) -> Result<String> {
    // 1. 使用明确定义的参数初始化 Argon2id。
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::default(), get_argon2_params());

    // 2. 执行哈希计算（新版本会自动生成盐值）。
    let password_hash = argon2.hash_password(password.as_bytes())
        .map_err(|e| AppError::Hash(format!("Failed to hash password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

/// Verifies a password against an Argon2id hash.
///
/// # Arguments
/// * `password` - The raw password to verify.
/// * `password_hash` - The stored PHC formatted hash.
///
/// # Returns
/// Returns `true` if the password matches the hash, `false` otherwise.
//
// // 使用 Argon2id 哈希验证密码。
// //
// // # 参数
// // * `password` - 要验证的原始密码。
// // * `password_hash` - 存储的 PHC 格式哈希。
// //
// // # 返回
// // 如果密码与哈希匹配则返回 `true`，否则返回 `false`。
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    // 1. 解析存储的哈希。
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Hash(format!("Invalid password hash format: {}", e)))?;

    // 2. 使用默认配置进行验证，参数会从 hash 字符串中自动加载。
    let argon2 = Argon2::default();
    let result = argon2.verify_password(password.as_bytes(), &parsed_hash);

    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Failed to hash password");

        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(password, &hash).expect("Failed to verify"));
        assert!(!verify_password("wrong_password", &hash).expect("Failed to verify"));
    }

    #[test]
    fn test_different_hashes() {
        let password = "same_password";
        let hash1 = hash_password(password).expect("Failed to hash");
        let hash2 = hash_password(password).expect("Failed to hash");

        assert_ne!(hash1, hash2, "Hashes should differ due to random salt");
        assert!(verify_password(password, &hash1).expect("Failed to verify hash1"));
        assert!(verify_password(password, &hash2).expect("Failed to verify hash2"));
    }

    #[test]
    fn test_invalid_hash_format() {
        let result = verify_password("any_password", "invalid_hash");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid password hash format"));
    }

    #[test]
    fn test_corrupted_hash() {
        let result = verify_password("password", "$argon2id$v=19$m=19456,t=2,p=1$corrupted");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_password() {
        let hash = hash_password("").expect("Should hash empty password");
        assert!(verify_password("", &hash).expect("Should verify empty password"));
        assert!(!verify_password("not_empty", &hash).expect("Should not match"));
    }
}
