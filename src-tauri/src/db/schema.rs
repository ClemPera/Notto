use rusqlite::{Connection, Result as SqliteResult};

/// Create all database tables
pub fn create_tables(conn: &Connection) -> SqliteResult<()> {
    // Users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            salt BLOB NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            user_id TEXT PRIMARY KEY,
            token TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // TOTP secrets table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS totp_secrets (
            user_id TEXT PRIMARY KEY,
            secret TEXT NOT NULL,
            verified INTEGER NOT NULL DEFAULT 0,
            backup_codes TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            parent_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Notes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            title TEXT NOT NULL,
            content BLOB NOT NULL,
            folder_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_encrypted INTEGER NOT NULL DEFAULT 1,
            sync_version INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(folder_id) REFERENCES folders(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Create FTS5 virtual table for full-text search on notes
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            note_id UNINDEXED,
            title,
            content,
            content=notes,
            content_rowid=rowid
        )",
        [],
    )?;

    // Sync metadata table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_metadata (
            note_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            last_sync TEXT,
            remote_revision TEXT,
            is_conflicted INTEGER NOT NULL DEFAULT 0,
            conflict_versions TEXT,
            FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Encryption parameters table (stores per-user parameters)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS encryption_params (
            user_id TEXT PRIMARY KEY,
            salt BLOB NOT NULL,
            argon2_memory INTEGER NOT NULL,
            argon2_iterations INTEGER NOT NULL,
            argon2_parallelism INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Recovery phrases table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recovery_phrases (
            user_id TEXT PRIMARY KEY,
            phrase_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_user ON notes(user_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_folders_user ON folders(user_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sync_metadata_user ON sync_metadata(user_id)",
        [],
    )?;

    Ok(())
}
