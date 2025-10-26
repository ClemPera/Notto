use std::{path::PathBuf, sync::Mutex};

use rusqlite::Connection;
use tauri_plugin_log::log::{debug, trace};

pub mod operations;
pub mod schema;

pub fn init(db_path: PathBuf) -> Result<Mutex<Connection>, Box<dyn std::error::Error>> {
    //TODO: add check if db is init to not recreate the tables, etc. 
    //      I think there's an easy way to handle that. and also db updates
    debug!("creating/opening database at {db_path:?}");

    let conn = Connection::open(db_path)?;

    trace!("db create correctly: {conn:?}");
    
    // Create tables
    schema::Note::create(&conn)?;
    trace!("Tables have been created correctly: {conn:?}");

    Ok(Mutex::new(conn))
}
