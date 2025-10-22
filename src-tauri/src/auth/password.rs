use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, PasswordHash};

/// Hash a password using Argon2id
pub fn hash_password(password: &str, salt_bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let salt = SaltString::encode_b64(salt_bytes)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    let argon2 = Argon2::default();
    // Pass password as bytes
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    Ok(password_hash.to_string())
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_hash = PasswordHash::new(hash_str)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;
    // Pass password as bytes
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let mut rng = rand::thread_rng();
        let salt_bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();

        let hash = hash_password(password, &salt_bytes).unwrap();
        assert!(verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let mut rng = rand::thread_rng();
        let salt_bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();

        let hash = hash_password(password, &salt_bytes).unwrap();
        assert!(verify_password(wrong_password, &hash).is_err());
    }

    #[test]
    fn test_same_password_different_salt_different_hash() {
        let password = "test_password";
        let mut rng = rand::thread_rng();
        let salt1: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();
        let salt2: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();

        let hash1 = hash_password(password, &salt1).unwrap();
        let hash2 = hash_password(password, &salt2).unwrap();

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);
        // But both should verify the same password
        assert!(verify_password(password, &hash1).is_ok());
        assert!(verify_password(password, &hash2).is_ok());
    }
}
