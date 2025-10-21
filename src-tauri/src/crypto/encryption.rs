use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use super::EncryptionKey;

pub const NONCE_LENGTH: usize = 12; // 96 bits for GCM
pub const TAG_LENGTH: usize = 16; // 128 bits for GCM

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Serialize to format: [nonce_length(1 byte) | nonce | ciphertext]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1 + self.nonce.len() + self.ciphertext.len());
        result.push(self.nonce.len() as u8);
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 2 {
            return Err("Encrypted data too short".into());
        }

        let nonce_len = data[0] as usize;
        if data.len() < 1 + nonce_len {
            return Err("Invalid nonce length".into());
        }

        let nonce = data[1..1 + nonce_len].to_vec();
        let ciphertext = data[1 + nonce_len..].to_vec();

        Ok(EncryptedData { nonce, ciphertext })
    }
}

/// Encrypt plaintext using AES-256-GCM
///
/// # Arguments
/// * `key` - 256-bit encryption key
/// * `plaintext` - Data to encrypt
/// * `additional_data` - Optional additional authenticated data (AAD)
///
/// # Returns
/// Encrypted data with nonce
pub fn encrypt(
    key: &EncryptionKey,
    plaintext: &[u8],
    additional_data: Option<&[u8]>,
) -> Result<EncryptedData, Box<dyn std::error::Error>> {
    // Generate random nonce (96 bits = 12 bytes)
    let mut rng = rand::thread_rng();
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())?;

    // Prepare payload with optional AAD
    let payload = match additional_data {
        Some(aad) => Payload {
            msg: plaintext,
            aad,
        },
        None => Payload {
            msg: plaintext,
            aad: b"",
        },
    };

    // Encrypt
    let ciphertext = cipher.encrypt(&nonce, payload)?;

    Ok(EncryptedData {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt ciphertext using AES-256-GCM
///
/// # Arguments
/// * `key` - 256-bit encryption key
/// * `encrypted_data` - Data to decrypt (with nonce)
/// * `additional_data` - Optional additional authenticated data (AAD) - must match encryption
///
/// # Returns
/// Decrypted plaintext
pub fn decrypt(
    key: &EncryptionKey,
    encrypted_data: &EncryptedData,
    additional_data: Option<&[u8]>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if encrypted_data.nonce.len() != NONCE_LENGTH {
        return Err("Invalid nonce length".into());
    }

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    nonce_bytes.copy_from_slice(&encrypted_data.nonce);
    let nonce = Nonce::from(nonce_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())?;

    // Prepare payload with optional AAD
    let payload = match additional_data {
        Some(aad) => Payload {
            msg: &encrypted_data.ciphertext,
            aad,
        },
        None => Payload {
            msg: &encrypted_data.ciphertext,
            aad: b"",
        },
    };

    // Decrypt
    let plaintext = cipher.decrypt(&nonce, payload)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::new([42u8; 32]);
        let plaintext = b"Hello, World! This is a test message.";

        let encrypted = encrypt(&key, plaintext, None).unwrap();
        let decrypted = decrypt(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = EncryptionKey::new([42u8; 32]);
        let plaintext = b"Secret message";
        let aad = b"note_id_12345";

        let encrypted = encrypt(&key, plaintext, Some(aad)).unwrap();
        let decrypted = decrypt(&key, &encrypted, Some(aad)).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_serialization() {
        let key = EncryptionKey::new([42u8; 32]);
        let plaintext = b"Test data for serialization";

        let encrypted = encrypt(&key, plaintext, None).unwrap();
        let serialized = encrypted.to_bytes();
        let deserialized = EncryptedData::from_bytes(&serialized).unwrap();
        let decrypted = decrypt(&key, &deserialized, None).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = EncryptionKey::new([42u8; 32]);
        let key2 = EncryptionKey::new([43u8; 32]);
        let plaintext = b"Secret message";

        let encrypted = encrypt(&key1, plaintext, None).unwrap();
        let result = decrypt(&key2, &encrypted, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = EncryptionKey::new([42u8; 32]);
        let plaintext = b"Secret message";
        let aad1 = b"note_id_12345";
        let aad2 = b"note_id_54321";

        let encrypted = encrypt(&key, plaintext, Some(aad1)).unwrap();
        let result = decrypt(&key, &encrypted, Some(aad2));

        assert!(result.is_err());
    }
}
