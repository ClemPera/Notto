use bip39::{Mnemonic, Language};
use sha2::{Sha256, Digest};

/// Generate a BIP39 recovery phrase from a password
///
/// The recovery phrase is deterministically derived from the password,
/// so the same password always produces the same recovery phrase.
pub fn generate_recovery_phrase(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Hash the password multiple times to create entropy
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    hasher.update(b"recovery_phrase_entropy");
    let hash2 = hasher.finalize();

    // Take first 32 bytes (256 bits) from combined hash for 24-word mnemonic
    let mut entropy = [0u8; 32];
    entropy[..16].copy_from_slice(&hash1[..16]);
    entropy[16..].copy_from_slice(&hash2[..16]);

    // Generate mnemonic (24 words)
    // In bip39 2.0, from_entropy takes only the entropy bytes
    let mnemonic = Mnemonic::from_entropy(&entropy)?;

    Ok(mnemonic.to_string())
}

/// Derive encryption key from recovery phrase
///
/// This allows account recovery using the recovery phrase
pub fn recovery_phrase_to_key(phrase: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mnemonic = Mnemonic::parse_in(Language::English, phrase)?;
    let entropy = mnemonic.to_entropy();

    // Hash entropy to create a consistent 32-byte key
    let mut hasher = Sha256::new();
    hasher.update(&entropy);
    let hash = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_phrase_generation() {
        let password = "test_password_123";
        let phrase = generate_recovery_phrase(password).unwrap();

        // Verify it's a valid BIP39 mnemonic
        let mnemonic = Mnemonic::parse_in(Language::English, &phrase);
        assert!(mnemonic.is_ok());

        // Should be 24 words
        let word_count = phrase.split_whitespace().count();
        assert_eq!(word_count, 24);
    }

    #[test]
    fn test_recovery_phrase_deterministic() {
        let password = "test_password_123";
        let phrase1 = generate_recovery_phrase(password).unwrap();
        let phrase2 = generate_recovery_phrase(password).unwrap();

        // Same password should produce same phrase
        assert_eq!(phrase1, phrase2);
    }

    #[test]
    fn test_recovery_phrase_to_key() {
        let password = "test_password_123";
        let phrase = generate_recovery_phrase(password).unwrap();
        let key = recovery_phrase_to_key(&phrase).unwrap();

        // Key should be 32 bytes
        assert_eq!(key.len(), 32);

        // Same phrase should produce same key
        let key2 = recovery_phrase_to_key(&phrase).unwrap();
        assert_eq!(key, key2);
    }
}
