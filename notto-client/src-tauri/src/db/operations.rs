use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::{crypt::{self, NoteData}, db::schema::{Note, User}};

pub fn create_note(conn: &Connection, title: String) -> Result<(), Box<dyn std::error::Error>> {
    //TODO: add mek
    // let note = crypt::encrypt_note(title, "blablalbla".to_string()).unwrap(); //Content empty because it's first note

    // note.insert(conn).unwrap();

    Ok(())
}

pub fn get_note(conn: &Connection, id: u32) -> Result<NoteData, Box<dyn std::error::Error>> {
    let note = Note::select(conn, id).unwrap();

    let decrypted_note = crypt::decrypt_note(note).unwrap();

    debug!("decrypted note is: {decrypted_note:?}");

    Ok(decrypted_note)
}

pub fn create_account(conn: &Connection, username: String, password: String) {
    let encryption_data = crypt::create_account(password);

    debug!("{encryption_data:?}");

    let user = User {
        id: None,
        username,
        master_encryption_key: encryption_data.master_encryption_key,
    };

    user.insert(&conn).unwrap();

    //TODO: store master_encryption_key in ram
    //TODO: send that to server
}

/// Execute when frontend load for the first time
pub fn init(conn: &Connection) {
    User::create(&conn).unwrap();
}