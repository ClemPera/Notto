use chrono::{DateTime, Utc};
use mysql_async::{Conn};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub title: String,
    pub content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: Option<DateTime<Utc>>
}

impl Note {
    pub fn create(&self, conn: &Conn){
    }

}