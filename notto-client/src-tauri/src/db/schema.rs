use aes_gcm::{Aes256Gcm, Key};
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

#[derive(Debug)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,

    //TODO: Do not store that in plain text but use give the user the possibility to use biometric to decrypt
    pub master_encryption_key: Key<Aes256Gcm>, 
}

impl User {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY,
                username TEXT,
                master_encryption_key BLOB
            )", 
            (), // empty list of parameters.
        ).unwrap();

        Ok(())
    }

    pub fn insert(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO user (id, username, master_encryption_key) VALUES (?1, ?2, ?3)", 
            (&self.id, &self.username, &self.master_encryption_key.to_vec())
        ).unwrap();

        Ok(())
    }

    pub fn select(conn: &Connection, id: u32) -> Result<User, Box<dyn std::error::Error>> {
        let user = conn.query_one(
            "SELECT id, username, master_encryption_key FROM user WHERE id = ?", 
            (id,),
            |row| {
                let mek: Vec<u8> = row.get(2)?;
                let mek: [u8; 32] = mek.try_into().unwrap();
                let mek: Key<Aes256Gcm> = mek.into();

                Ok(User{
                    id: row.get(0)?,
                    username: row.get(1)?,
                    master_encryption_key: mek,
                })
            }
        ).unwrap();

        Ok(user)
    }

    pub fn select_all(conn: &Connection) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT * FROM user").unwrap();

        let rows = stmt.query_map(
            [],
            |row| {
                let mek: Vec<u8> = row.get(2)?;
                let mek: [u8; 32] = mek.try_into().unwrap();
                let mek: Key<Aes256Gcm> = mek.into();

                Ok(User{
                    id: row.get(0)?,
                    username: row.get(1)?,
                    master_encryption_key: mek,
                })
            }
        ).unwrap();

        let mut users = Vec::new();

        for user in rows {
            users.push(user.unwrap());
        }

        Ok(users)
    }
    
    pub fn update(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        Err(Box::from("Not implemented yet"))
    }
}