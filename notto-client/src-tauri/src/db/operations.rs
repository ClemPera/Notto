use aes_gcm::{Aes256Gcm, Key};
use chrono::{DateTime, Local, NaiveDateTime};
use rusqlite::Connection;
use serde::Serialize;
use tauri_plugin_log::log::debug;

use crate::{crypt::{self, NoteData}, db::schema::{Note, User}};


pub fn create_note(conn: &Connection, id_user: u32, title: String, mek: Key<Aes256Gcm>) -> Result<(), Box<dyn std::error::Error>> {
    let (content, nonce) = crypt::encrypt_note("blablalbla".to_string(), mek).unwrap(); //Content empty because it's first note

    let note = Note {
        id: None,
        id_user: Some(id_user),
        content,
        nonce,
        title,
        updated_at: Local::now().naive_utc()
    };

    note.insert(conn,).unwrap();

    Ok(())
}

pub fn get_note(conn: &Connection, id: u32, mek: Key<Aes256Gcm>) -> Result<NoteData, Box<dyn std::error::Error>> {
    let note = Note::select(conn, id).unwrap();

    let decrypted_note = crypt::decrypt_note(note, mek).unwrap();

    debug!("decrypted note is: {decrypted_note:?}");

    Ok(decrypted_note)
}

pub fn get_notes(conn: &Connection, id_user: u32) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
    let notes = Note::select_all(conn, id_user).unwrap();

    Ok(notes)
}

pub fn update_note(conn: &Connection, note_data: NoteData, mek: Key<Aes256Gcm>) -> Result<(), Box<dyn std::error::Error>> {
    let (content, nonce) = crypt::encrypt_note(note_data.content, mek).unwrap();
    
    let note = Note {
        id: Some(note_data.id),
        username: None,
        title: note_data.title,
        content,
        nonce,
        updated_at: note_data.updated_at,
    };
    
    note.update(conn).unwrap();
    
    Ok(())
}

pub fn create_user(conn: &Connection, username: String) -> Result<User, Box<dyn std::error::Error>> {
    let user_encryption_data = crypt::create_user();

    debug!("{user_encryption_data:?}");

    let user = User {
        id: None,
        username,
        master_encryption_key: user_encryption_data.master_encryption_key,
        salt_recovery_data: user_encryption_data.salt_recovery_data.to_string(),
        mek_recovery_nonce: user_encryption_data.mek_recovery_nonce,
        encrypted_mek_recovery: user_encryption_data.encrypted_mek_recovery,
        token: None
    };

    user.insert(&conn).unwrap();

    //TODO: send recovery keys to frontend

    Ok(user)
}

pub fn update_user(conn: &Connection, new_user: User) {
    new_user.update(conn).unwrap();
}

pub fn get_user(conn: &Connection, username: String) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let user = User::select(conn, username).unwrap();

    Ok(user)
}

pub fn get_users(conn: &Connection) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let users = User::select_all(conn).unwrap();

    Ok(users)
}

/// Execute when frontend load for the first time
pub fn init(conn: &Connection) {
}