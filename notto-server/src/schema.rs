#[derive(Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub title: String,
    pub content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: Option<DateTime<Utc>>
}

impl Note {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    
    }

}