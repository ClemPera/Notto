use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::{crypt::{self, NoteData}, db::schema::Note};

pub fn create_note(conn: &Connection, title: String) -> Result<(), Box<dyn std::error::Error>> {
    let note = crypt::encrypt_note(title, "blablalbla".to_string()).unwrap(); //Content empty because it's first note

    note.insert(conn).unwrap();

    Ok(())
}

pub fn get_note(conn: &Connection, id: u32) -> Result<NoteData, Box<dyn std::error::Error>> {
    let note = Note::select(conn, id).unwrap();

    let decrypted_note = crypt::decrypt_note(note).unwrap();

    debug!("decrypted note is: {decrypted_note:?}");

    Ok(decrypted_note)
}