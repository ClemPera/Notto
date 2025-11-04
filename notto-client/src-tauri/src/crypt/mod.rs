use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce, aead::Aead, aes::Aes256};
use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use bip39::Language;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri_plugin_log::log::{debug, info};

use crate::db::schema;

#[derive(Debug)]
pub struct NoteData {
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct EncryptionData {
    pub master_encryption_key: Key<Aes256Gcm>,

    pub recovery_key_auth: String,
    pub recovery_key_data: String,

    pub salt_auth: SaltString,
    pub salt_data: SaltString,
    pub salt_recovery_auth: SaltString,
    pub salt_recovery_data: SaltString,
    pub salt_server_auth: SaltString,
    pub salt_server_recovery: SaltString,

    pub mek_password_nonce: Vec<u8>,
    pub mek_recovery_nonce: Vec<u8>,

    pub encrypted_mek_password: Vec<u8>,
    pub encrypted_mek_recovery: Vec<u8>,

    pub stored_password_hash: String,
    pub stored_recovery_hash: String,
}

pub fn create_account(password: String) -> EncryptionData {
    //Generate encryption key
    // let master_encryption_key: &[u8; 32] = &[200, 177, 198, 105, 203, 59, 243, 159, 130, 46, 182, 8, 195, 75, 214, 236, 236, 168, 29, 157, 56, 167, 96, 197, 28, 42, 245, 123, 65, 211, 59, 54];
    let master_encryption_key: Key<Aes256Gcm> = Aes256Gcm::generate_key(OsRng).into();

    //Generate recovery keys for auth and data
    let recovery_key_auth = bip39::Mnemonic::generate_in(Language::English, 24)
        .unwrap()
        .to_string();
    let recovery_key_data = bip39::Mnemonic::generate_in(Language::English, 24)
        .unwrap()
        .to_string();

    //Init AesGcm and Argon2
    let argon2 = Argon2::default();
    let cipher = Aes256Gcm::new(&master_encryption_key);

    //Generate needed salts
    let salt_auth = SaltString::generate(&mut OsRng);
    let salt_data = SaltString::generate(&mut OsRng);
    let salt_recovery_auth = SaltString::generate(&mut OsRng);
    let salt_recovery_data = SaltString::generate(&mut OsRng);
    let salt_server_auth = SaltString::generate(&mut OsRng);
    let salt_server_recovery = SaltString::generate(&mut OsRng);

    //Generate hash for password and data
    let password_hash_auth = argon2
        .hash_password(password.as_bytes(), &salt_auth)
        .unwrap()
        .to_string();
    let recovery_hash_auth = argon2
        .hash_password(recovery_key_auth.as_bytes(), &salt_recovery_auth)
        .unwrap()
        .to_string();
    let password_hash_data = argon2
        .hash_password(password.as_bytes(), &salt_data)
        .unwrap()
        .to_string();
    let recovery_hash_data = argon2
        .hash_password(recovery_key_data.as_bytes(), &salt_recovery_data)
        .unwrap()
        .to_string();

    //Generate nonce for mek password/recovery
    let mek_password_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mek_recovery_nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    //Generate hash for mek password and recovery
    let encrypted_mek_password = cipher
        .encrypt(&mek_password_nonce, password_hash_data.as_bytes())
        .unwrap();
    let encrypted_mek_recovery = cipher
        .encrypt(&mek_recovery_nonce, recovery_hash_data.as_bytes())
        .unwrap();

    //Generate hashs for password store on server
    let stored_password_hash = argon2
        .hash_password(password_hash_auth.as_bytes(), &salt_server_auth)
        .unwrap()
        .to_string();
    let stored_recovery_hash = argon2
        .hash_password(recovery_hash_auth.as_bytes(), &salt_server_recovery)
        .unwrap()
        .to_string();

    EncryptionData {
        master_encryption_key,
        recovery_key_auth,
        recovery_key_data,
        salt_auth,
        salt_data,
        salt_recovery_auth,
        salt_recovery_data,
        salt_server_auth,
        salt_server_recovery,
        mek_password_nonce: mek_password_nonce.to_vec(),
        mek_recovery_nonce:  mek_recovery_nonce.to_vec(),
        encrypted_mek_password,
        encrypted_mek_recovery,
        stored_password_hash,
        stored_recovery_hash,
    }
}

pub fn encrypt_note(
    content: String,
    master_encryption_key: Key<Aes256Gcm>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    //Encrypt
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes256Gcm::new(&master_encryption_key);

    let ciphertext = cipher.encrypt(&nonce, content.as_bytes()).unwrap();

    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_note(note: schema::Note, mek: Key<Aes256Gcm>) -> Result<NoteData, Box<dyn std::error::Error>> {
    let nonce_array: [u8; 12] = note.nonce.try_into().expect("nonce must be 12 bytes");
    let nonce = Nonce::from(nonce_array);

    let cipher = Aes256Gcm::new(&mek);
    let plaintext = cipher.decrypt(&nonce, note.content.as_ref()).unwrap();
    let data_unser = NoteData {
        title: note.title,
        content: String::from_utf8(plaintext).unwrap(),
        created_at: note.created_at.unwrap(),
    };

    Ok(data_unser)
}
