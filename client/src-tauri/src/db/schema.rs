use chrono::{DateTime, Utc};
use rusqlite::Connection;
use tauri_plugin_log::log::debug;

#[derive(Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub title: String,
    pub content: Vec<u8>, //Serialized encrypted content.
    pub nonce: Vec<u8>, //Nonce used to decrypt data.?
    pub created_at: Option<DateTime<Utc>>
}

impl Note {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS note (
            id INTEGER PRIMARY KEY,
            title TEXT,
            content BLOB,
            nonce BLOB,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )", 
            (), // empty list of parameters.
        ).unwrap();

        Ok(())
    }

    pub fn insert(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        debug!("insert new note with vals: {:?}, {:?}, {:?}", &self.title, &self.content, &self.nonce);
        conn.execute(
            "INSERT INTO note (title, content, nonce) VALUES (?1, ?2, ?3)", 
            (&self.title, &self.content, &self.nonce)
        ).unwrap();

        Ok(())
    }

    pub fn select(conn: &Connection, id: u32) -> Result<Note, Box<dyn std::error::Error>> {
        let note = conn.query_one(
            "SELECT id, title, content, nonce, created_at FROM note WHERE id = ?", 
            (id,),
            |row| {
                Ok(Note{
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    nonce: row.get(3)?,
                    created_at: row.get(4)?
                })
            }
        ).unwrap();

        Ok(note)
    }
    
    pub fn update(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        Err(Box::from("Not implemented yet, consider versioning"))
    }
}