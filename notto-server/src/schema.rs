use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use mysql_async::{Conn, FromRowError, Row, params, prelude::{FromRow, Queryable, WithParams}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub id_user: u32,
    pub title: String,
    pub content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: NaiveDateTime,
}

impl FromRow for Note {
    fn from_row_opt(row: Row) -> Result<Self, FromRowError> {
        Ok(Note {
            id: row.get(0).ok_or(FromRowError(row.clone()))?,
            id_user: row.get(1).ok_or(FromRowError(row.clone()))?,
            title: row.get(2).ok_or(FromRowError(row.clone()))?,
            content: row.get(3).ok_or(FromRowError(row.clone()))?,
            nonce: row.get(4).ok_or(FromRowError(row.clone()))?,
            created_at: row.get(5).ok_or(FromRowError(row.clone()))?,
        })
    }
}

impl Note {
    pub async fn create(&self, conn: &mut Conn) {
        conn.exec_drop("INSERT INTO note (id_user, title, content, nonce, created_at) 
            VALUES (:id_user, :title, :content, :nonce, :created_at)", 
            params!(
                "id_user" => &self.id_user,
                "title" => &self.title,
                "content" => &self.content,
                "nonce" => &self.nonce,
                "created_at" => &self.created_at
            )).await.unwrap();
    }

    pub async fn update(&self, conn: &mut Conn) {
        conn.exec_drop("UPDATE note 
            SET (title = :title, content = :content, nonce = :nonce) 
            WHERE id = :id", 
            params!(
                "title" => &self.title,
                "content" => &self.content,
                "nonce" => &self.nonce
            )).await.unwrap();
    }

    pub async fn select_all(conn: &mut Conn, id_user: u32) -> Vec<Note> {
        conn.exec("SELECT id, id_user, title, content, nonce, created_at 
            FROM note WHERE id_user = :id_user",
            params!(
                "id_user" => id_user
            )).await.unwrap()
    }
}