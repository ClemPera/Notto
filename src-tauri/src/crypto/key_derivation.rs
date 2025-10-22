use argon2::{
    password_hash::{SaltString, PasswordHasher},
    Argon2, Params, Version, Algorithm,
};
use rand::Rng;
use sha2::{Sha256, Digest};

pub const KEY_LENGTH: usize = 32; // 256 bits for AES-256

#[derive(Debug, Clone)]
pub struct EncryptionKey {
    pub key: [u8; KEY_LENGTH],
}

impl EncryptionKey {
    pub fn new(key_bytes: [u8; KEY_LENGTH]) -> Self {
        Self { key: key_bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
}

/// Derive encryption key from password using Argon2id
///
/// # Arguments
/// * `password` - User's password
/// * `salt` - Random salt (16 bytes recommended). If None, generates random salt
/// * `memory` - Memory cost in KiB (default: 19456 = ~19 MB)
/// * `iterations` - Time cost (default: 2)
/// * `parallelism` - Parallelism degree (default: 1)
///
/// # Returns
/// Tuple of (derived_key, salt_bytes)
pub fn derive_key(
    password: &str,
    salt: Option<&[u8]>,
    memory: Option<u32>,
    iterations: Option<u32>,
    parallelism: Option<u32>,
) -> Result<(EncryptionKey, Vec<u8>), Box<dyn std::error::Error>> {
    let memory = memory.unwrap_or(19456);
    let iterations = iterations.unwrap_or(2);
    let parallelism = parallelism.unwrap_or(1);

    // Generate or use provided salt
    let salt_bytes = if let Some(s) = salt {
        s.to_vec()
    } else {
        let mut rng = rand::thread_rng();
        (0..16).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>()
    };

    // Create Argon2id hasher with parameters
    let params = Params::new(memory, iterations, parallelism, Some(KEY_LENGTH))
        .map_err(|e| format!("Failed to create Argon2 params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Create salt string from bytes
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;

    // Hash password to get the derived key
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    // Extract the actual hash bytes
    let hash_str = password_hash.hash.ok_or("Failed to extract hash")?;
    let hash_bytes = hash_str.as_bytes();

    // Take first 32 bytes for AES-256 key
    let mut key = [0u8; KEY_LENGTH];
    if hash_bytes.len() >= KEY_LENGTH {
        key.copy_from_slice(&hash_bytes[..KEY_LENGTH]);
    } else {
        // If hash is shorter than needed, use it as seed for SHA256
        let mut hasher = Sha256::new();
        hasher.update(hash_bytes);
        hasher.update(password.as_bytes());
        hasher.update(&salt_bytes);
        let result = hasher.finalize();
        key.copy_from_slice(&result[..KEY_LENGTH]);
    }

    Ok((EncryptionKey::new(key), salt_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let (key1, salt) = derive_key(password, None, None, None, None).unwrap();
        let (key2, _) = derive_key(password, Some(&salt), None, None, None).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1.key, key2.key);
    }

    #[test]
    fn test_different_salts_different_keys() {
        let password = "test_password_123";
        let (key1, _salt1) = derive_key(password, None, None, None, None).unwrap();
        let (key2, _salt2) = derive_key(password, None, None, None, None).unwrap();

        // Different random salts should produce different keys
        assert_ne!(key1.key, key2.key);
    }
}
