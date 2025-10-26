use rusqlite::Connection;

pub struct Note {
    pub id: Option<u32>,
    pub data: String //Contain serialized encrypted data (including content and title).
}

impl Note {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS note (
            id INTEGER PRIMARY KEY,
            data BLOB
            )", 
            (), // empty list of parameters.
        )?;

        Ok(())
    }

    pub fn insert(self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO note (data) VALUES (?1)", 
            (&self.data,)
        )?;

        Ok(())
    }
}