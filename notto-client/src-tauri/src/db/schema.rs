use aes_gcm::{Aes256Gcm, Key};
use chrono::NaiveDateTime;
use rusqlite::Connection;
use tauri_plugin_log::log::debug;

use crate::crypt::NoteData;

#[derive(Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub id_user: Option<u32>,
    pub title: String,
    pub content: Vec<u8>, //Serialized encrypted content.
    pub nonce: Vec<u8>, //Nonce used to decrypt data.?
    pub created_at: Option<NaiveDateTime>
}

impl Note {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS note (
                id INTEGER PRIMARY KEY,
                id_user INTEGER NOT NULL REFERENCES user(id),
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
            "INSERT INTO note (title, content, nonce, id_user) VALUES (?1, ?2, ?3, ?4)", 
            (&self.title, &self.content, &self.nonce, &self.id_user)
        ).unwrap();

        Ok(())
    }

    pub fn select(conn: &Connection, id: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let note = conn.query_one(
            "SELECT * FROM note WHERE id = ?", 
            (id,),
            |row| {
                Ok(Note{
                    id: row.get(0)?,
                    id_user: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    nonce: row.get(4)?,
                    created_at: row.get(5)?
                })
            }
        ).unwrap();

        Ok(note)
    }

    pub fn select_all(conn: &Connection, id_user: u32) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT * FROM note WHERE id_user = ?").unwrap();

        let rows = stmt.query_map(
            [id_user,],
            |row| {
                Ok(Note{
                    id: row.get(0)?,
                    id_user: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    nonce: row.get(4)?,
                    created_at: row.get(5)?,
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
        conn.execute("UPDATE note SET title = ?, content = ?, nonce = ? WHERE id = ?",
            (&self.title, &self.content, &self.nonce, &self.id))?;

        Ok(())
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

    pub fn select(conn: &Connection, id: u32) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn select_all(conn: &Connection) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
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