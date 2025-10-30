use std::{path::PathBuf, sync::Mutex};

use rusqlite::Connection;
use tauri_plugin_log::log::{debug, trace};

pub mod operations;
pub mod schema;

pub fn init(db_path: PathBuf) -> Result<Mutex<Connection>, Box<dyn std::error::Error>> {
    debug!("creating/opening database at {db_path:?}");
    let conn = Connection::open(db_path)?;
    trace!("db create correctly: {conn:?}");
    
    // Create tables
    schema::Note::create(&conn)?;
    trace!("Tables have been created correctly");

    Ok(Mutex::new(conn))
}
