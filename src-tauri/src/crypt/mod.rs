use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString
}, Argon2};
use tauri_plugin_log::log::info;

use crate::db::schema;

pub fn note(title: String, content: String) -> Result<schema::Note, Box<dyn std::error::Error>> {
    // let argon2 = Argon2::default();

    //TODO: 
    let pass = b"hunter2";

    // let salt_auth = SaltString::generate(&mut OsRng);
    // let salt_data = SaltString::generate(&mut OsRng);
    let salt_auth = "1XBSohT3A0PR4uCqkZ+tAw";
    let salt_data = "CpB9QyFaHVZkyfW2zPz+KA";
    // let auth_hash = argon2.hash_password(pass, &salt_auth)?.to_string();
    // let data_hash = argon2.hash_password(pass, &salt_data)?.to_string();
    let auth_hash = "$argon2id$v=19$m=19456,t=2,p=1$1XBSohT3A0PR4uCqkZ+tAw$3d5B9gbwvtI7hDt/GpRRJkUkz5ktng+DuqtchIG4rpw";
    let data_hash = "$argon2id$v=19$m=19456,t=2,p=1$CpB9QyFaHVZkyfW2zPz+KA$UTQgJFWjIyHQpEg8jdO02R2dD8+YOTgc+jRgaePiUQ0"

    let note = schema::Note {
        id: None,
        data: "yo".to_string()
    };
    
    Ok(note)
}