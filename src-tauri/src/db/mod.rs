pub mod schema;
pub mod operations;

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub type DbConnection = Arc<Mutex<Connection>>;

/// Initialize the database connection and create schema
pub fn init(db_path: &PathBuf) -> SqliteResult<DbConnection> {
    let conn = Connection::open(db_path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // Enable WAL mode for better concurrency
    conn.execute("PRAGMA journal_mode = WAL;", [])?;

    // Create schema
    schema::create_tables(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}

/// Get a connection guard from the pool
pub fn get_conn(db: &DbConnection) -> rusqlite::Result<std::sync::MutexGuard<Connection>> {
    db.lock().map_err(|e| rusqlite::Error::ToSqlConversionFailure(
        Box::new(std::fmt::Error),
        std::borrow::Cow::from("Failed to acquire database lock"),
    ))
}
