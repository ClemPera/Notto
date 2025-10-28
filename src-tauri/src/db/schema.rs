use rusqlite::Connection;

#[derive(Debug)]
pub struct Note {
    pub id: Option<u32>,
    pub data: Vec<u8>, //Contain serialized encrypted data (including content and title).
    pub nonce: Vec<u8> //Nonce used to decrypt data
}

impl Note {
    pub fn create(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS note (
            id INTEGER PRIMARY KEY,
            data BLOB,
            nonce BLOB
            )", 
            (), // empty list of parameters.
        )?;

        Ok(())
    }

    pub fn insert(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO note (data, nonce) VALUES (?1, ?2)", 
            (&self.data, &self.nonce)
        )?;

        Ok(())
    }

    pub fn select(conn: &Connection, id: u32) -> Result<Note, Box<dyn std::error::Error>> {
        let note = conn.query_one(
            "SELECT id, data, nonce FROM note WHERE id = ?", 
            (id,),
            |row| {
                Ok(Note{
                    id: row.get(0)?,
                    data: row.get(1)?,
                    nonce: row.get(2)?,
                })
            }
        )?;

        Ok(note)
    }
}