use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::db::schema;


pub fn create_note(conn: &Connection, title: String) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{title:?}");

    let note = schema::Note {
        id: None,
        data: title,
    };

    note.insert(conn)?;

    Ok(())
}