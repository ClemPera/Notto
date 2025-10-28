use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::{crypt::{self, NoteData}, db::schema::Note};

pub fn create_note(conn: &Connection, title: String) -> Result<(), Box<dyn std::error::Error>> {
    let note = crypt::encrypt_note(title, "".to_string())?; //Content empty because it's first note

    note.insert(conn)?;

    Ok(())
}

pub fn get_note(conn: &Connection, id: u32) -> Result<NoteData, Box<dyn std::error::Error>> {
    let note = Note::select(conn, id)?;

    let decrypted_note = crypt::decrypt_note(note)?;

    debug!("decrypted note is: {decrypted_note:?}");

    Ok(decrypted_note)
}