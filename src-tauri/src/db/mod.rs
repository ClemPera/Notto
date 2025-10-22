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
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    // Create schema
    schema::create_tables(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}

/// Get a connection guard from the pool
pub fn get_conn(db: &DbConnection) -> rusqlite::Result<std::sync::MutexGuard<Connection>> {
    db.lock().map_err(|_e| {
        // In rusqlite 0.37, ToSqlConversionFailure takes only one argument (a Box<dyn Error>)
        rusqlite::Error::ToSqlConversionFailure(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire database lock"
            ))
        )
    })
}
