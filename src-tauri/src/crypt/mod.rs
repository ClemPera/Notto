use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use argon2::{password_hash::{rand_core::{OsRng, RngCore}, PasswordHash, PasswordHasher, PasswordVerifier, SaltString
}, Argon2};
use chrono::{DateTime, Utc};
use tauri_plugin_log::log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::db::schema;

#[derive(Debug)]
pub struct NoteData {
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>
}

pub fn encrypt_note(title: String, content: String) -> Result<schema::Note, Box<dyn std::error::Error>> {
    // let argon2 = Argon2::default();

    //TODO: 
    // let pass = b"hunter2";

    // let salt_auth = SaltString::generate(&mut OsRng);
    // let salt_data = SaltString::generate(&mut OsRng);
    // let salt_auth = "1XBSohT3A0PR4uCqkZ+tAw";
    // let salt_data = "CpB9QyFaHVZkyfW2zPz+KA";
    // let auth_hash = argon2.hash_password(pass, &salt_auth)?.to_string();
    // let data_hash = argon2.hash_password(pass, &salt_data)?.to_string();
    // let auth_hash = "$argon2id$v=19$m=19456,t=2,p=1$1XBSohT3A0PR4uCqkZ+tAw$3d5B9gbwvtI7hDt/GpRRJkUkz5ktng+DuqtchIG4rpw";
    // let data_hash = "$argon2id$v=19$m=19456,t=2,p=1$CpB9QyFaHVZkyfW2zPz+KA$UTQgJFWjIyHQpEg8jdO02R2dD8+YOTgc+jRgaePiUQ0";

    // let master_encryption_key = Aes256Gcm::generate_key(OsRng);

    //Encrypt
    let master_encryption_key: &[u8; 32] = &[200, 177, 198, 105, 203, 59, 243, 159, 130, 46, 182, 8, 195, 75, 214, 236, 236, 168, 29, 157, 56, 167, 96, 197, 28, 42, 245, 123, 65, 211, 59, 54];
    let master_encryption_key: &Key<Aes256Gcm> = master_encryption_key.into();
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let cipher = Aes256Gcm::new(&master_encryption_key);

    let ciphertext = cipher.encrypt(&nonce, content.as_bytes()).unwrap();

    let note = schema::Note {
        id: None,
        title: title,
        content: ciphertext,
        nonce: nonce.to_vec(),
        created_at: None
    };
    
    Ok(note)
}

pub fn decrypt_note(note: schema::Note) -> Result<NoteData, Box<dyn std::error::Error>> {
    let master_encryption_key: &[u8; 32] = &[200, 177, 198, 105, 203, 59, 243, 159, 130, 46, 182, 8, 195, 75, 214, 236, 236, 168, 29, 157, 56, 167, 96, 197, 28, 42, 245, 123, 65, 211, 59, 54];
    let master_encryption_key: &Key<Aes256Gcm> = master_encryption_key.into();
    
    let nonce_array: [u8; 12] = note.nonce.try_into().expect("nonce must be 12 bytes");
    let nonce = Nonce::from(nonce_array);

    let cipher = Aes256Gcm::new(&master_encryption_key);
    let plaintext = cipher.decrypt(&nonce, note.content.as_ref()).unwrap();
    let data_unser = NoteData {
        title: note.title,
        content: String::from_utf8(plaintext).unwrap(),
        created_at: note.created_at.unwrap()
    };
    
    Ok(data_unser)
}