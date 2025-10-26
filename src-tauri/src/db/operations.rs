use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::crypt;

pub fn create_note(conn: &Connection, title: String) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{title:?}");

    // let note = schema::Note {
    //     id: None,
    //     data: title,
    // };

    let note = crypt::note(title, "yo".to_string())?;

    note.insert(conn)?;

    Ok(())
}