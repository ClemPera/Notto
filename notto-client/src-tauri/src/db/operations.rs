use aes_gcm::{Aes256Gcm, Key};
use rusqlite::Connection;
use serde::Serialize;
use tauri_plugin_log::log::debug;

use crate::{crypt::{self, NoteData}, db::schema::{Note, User}};

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: u32,
    pub username: String,
}

pub fn create_note(conn: &Connection, title: String, mek: Key<Aes256Gcm>) -> Result<(), Box<dyn std::error::Error>> {
    //TODO: add mek
    let note = crypt::encrypt_note(title, "blablalbla".to_string(), mek).unwrap(); //Content empty because it's first note

    note.insert(conn).unwrap();

    Ok(())
}

pub fn get_note(conn: &Connection, id: u32, mek: Key<Aes256Gcm>) -> Result<NoteData, Box<dyn std::error::Error>> {
    let note = Note::select(conn, id).unwrap();

    let decrypted_note = crypt::decrypt_note(note, mek).unwrap();

    debug!("decrypted note is: {decrypted_note:?}");

    Ok(decrypted_note)
}

pub fn create_account(conn: &Connection, username: String, password: String) -> Result<User, Box<dyn std::error::Error>> {
    let encryption_data = crypt::create_account(password);

    debug!("{encryption_data:?}");

    let user = User {
        id: None,
        username,
        master_encryption_key: encryption_data.master_encryption_key,
    };

    user.insert(&conn).unwrap();

    Ok(user)

    //TODO: send that to server
}

pub fn get_users(conn: &Connection) -> Result<Vec<FilteredUser>, Box<dyn std::error::Error>> {
    let users = User::select_all(conn).unwrap();

    let filtered_users = users.iter().map(|u| FilteredUser {
        id: u.id.unwrap(),
        username: u.username.to_owned()
    }).collect();
    
    Ok(filtered_users)
}

/// Execute when frontend load for the first time
pub fn init(conn: &Connection) {
    User::create(&conn).unwrap();
}