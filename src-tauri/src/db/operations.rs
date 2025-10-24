use crate::models::*;
use chrono::Utc;
use rusqlite::{params, Connection, Result as SqliteResult};
use tauri_plugin_log::log::{info, warn};

// ==================== Note Operations ====================

pub fn create_note(
    conn: &Connection,
    user_id: &str,
    title: &str,
    content: &[u8],
    folder_id: Option<&str>,
) -> SqliteResult<String> {
    let note_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO notes (id, user_id, title, content, folder_id, created_at, updated_at, is_encrypted, sync_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 0)",
        params![&note_id, user_id, title, content, folder_id, &now, &now],
    )?;


    Ok(note_id)
}

pub fn get_note(conn: &Connection, note_id: &str) -> SqliteResult<Option<Vec<u8>>> {
    let mut stmt = conn.prepare("SELECT content FROM notes WHERE id = ?1")?;
    let content = stmt
        .query_row([note_id], |row| row.get::<_, Vec<u8>>(0))
        .ok();
    Ok(content)
}

pub fn update_note(
    conn: &Connection,
    note_id: &str,
    title: &str,
    content: &[u8],
) -> SqliteResult<()> {
    warn!("noteid1:{note_id:?}");

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3, sync_version = sync_version + 1
         WHERE id = ?4",
        params![title, content, &now, note_id],
    )?;
    Ok(())
}

pub fn delete_note(conn: &Connection, note_id: &str) -> SqliteResult<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])?;
    // Sync metadata is cascade deleted
    Ok(())
}

pub fn list_notes(
    conn: &Connection,
    user_id: &str,
    folder_id: Option<&str>,
) -> SqliteResult<Vec<String>> {
    // Handle two separate query paths to avoid lifetime issues with params macro
    if let Some(folder) = folder_id {
        let query =
            "SELECT id FROM notes WHERE user_id = ?1 AND folder_id = ?2 ORDER BY updated_at DESC";
        let mut stmt = conn.prepare(query)?;
        let note_ids = stmt
            .query_map(params![user_id, folder], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(note_ids)
    } else {
        let query = "SELECT id FROM notes WHERE user_id = ?1 AND folder_id IS NULL ORDER BY updated_at DESC";
        let mut stmt = conn.prepare(query)?;
        let note_ids = stmt
            .query_map(params![user_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(note_ids)
    }
}

// ==================== Folder Operations ====================

pub fn create_folder(
    conn: &Connection,
    user_id: &str,
    name: &str,
    parent_id: Option<&str>,
) -> SqliteResult<String> {
    let folder_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO folders (id, user_id, name, parent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&folder_id, user_id, name, parent_id, &now, &now],
    )?;

    Ok(folder_id)
}

pub fn list_folders(conn: &Connection, user_id: &str) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM folders WHERE user_id = ?1 ORDER BY name")?;
    let folder_ids = stmt
        .query_map([user_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(folder_ids)
}

// ==================== User Operations ====================

pub fn create_user(
    conn: &Connection,
    username: &str,
    password_hash: &str,
    salt: &[u8],
) -> SqliteResult<String> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO users (id, username, password_hash, salt, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&user_id, username, password_hash, salt, &now],
    )?;

    Ok(user_id)
}

pub fn get_user_by_username(
    conn: &Connection,
    username: &str,
) -> SqliteResult<Option<(String, String, Vec<u8>)>> {
    let mut stmt = conn.prepare("SELECT id, password_hash, salt FROM users WHERE username = ?1")?;
    let result = stmt
        .query_row([username], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .ok();

    Ok(result)
}

// ==================== Session Operations ====================

pub fn create_session(
    conn: &Connection,
    user_id: &str,
    token: &str,
    expires_in_hours: i64,
) -> SqliteResult<()> {
    let created_at = Utc::now();
    let expires_at = created_at + chrono::Duration::hours(expires_in_hours);

    conn.execute(
        "INSERT OR REPLACE INTO sessions (user_id, token, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            user_id,
            token,
            created_at.to_rfc3339(),
            expires_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

pub fn get_session(conn: &Connection, token: &str) -> SqliteResult<Option<String>> {
    let now = Utc::now().to_rfc3339();
    let mut stmt =
        conn.prepare("SELECT user_id FROM sessions WHERE token = ?1 AND expires_at > ?2")?;
    let user_id = stmt
        .query_row(params![token, &now], |row| row.get::<_, String>(0))
        .ok();

    Ok(user_id)
}

pub fn delete_session(conn: &Connection, user_id: &str) -> SqliteResult<()> {
    conn.execute("DELETE FROM sessions WHERE user_id = ?1", params![user_id])?;
    Ok(())
}

// ==================== Encryption Params Operations ====================

pub fn set_encryption_params(
    conn: &Connection,
    user_id: &str,
    params: &EncryptionParams,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO encryption_params
         (user_id, salt, argon2_memory, argon2_iterations, argon2_parallelism)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            user_id,
            &params.salt,
            params.argon2_memory,
            params.argon2_iterations,
            params.argon2_parallelism,
        ],
    )?;

    Ok(())
}

pub fn get_encryption_params(
    conn: &Connection,
    user_id: &str,
) -> SqliteResult<Option<EncryptionParams>> {
    let mut stmt = conn.prepare(
        "SELECT salt, argon2_memory, argon2_iterations, argon2_parallelism
         FROM encryption_params WHERE user_id = ?1",
    )?;
    let result = stmt
        .query_row([user_id], |row| {
            Ok(EncryptionParams {
                salt: row.get(0)?,
                argon2_memory: row.get(1)?,
                argon2_iterations: row.get(2)?,
                argon2_parallelism: row.get(3)?,
            })
        })
        .ok();

    Ok(result)
}

// ==================== Recovery Phrase Operations ====================

pub fn set_recovery_phrase_hash(
    conn: &Connection,
    user_id: &str,
    phrase_hash: &str,
) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO recovery_phrases (user_id, phrase_hash, created_at)
         VALUES (?1, ?2, ?3)",
        params![user_id, phrase_hash, &now],
    )?;

    Ok(())
}

pub fn get_recovery_phrase_hash(conn: &Connection, user_id: &str) -> SqliteResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT phrase_hash FROM recovery_phrases WHERE user_id = ?1")?;
    let hash = stmt
        .query_row([user_id], |row| row.get::<_, String>(0))
        .ok();

    Ok(hash)
}
