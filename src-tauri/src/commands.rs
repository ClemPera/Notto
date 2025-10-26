use serde::Serialize;
use tauri::State;

use crate::AppState;
use crate::db::operations;

///Convert any error to string for frontend
#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

impl From<Box<dyn std::error::Error>> for CommandError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}



#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn create_note(state: State<'_, AppState>, title: String) -> Result<(), CommandError> {
    let conn = state.database.lock().unwrap();
    
    operations::create_note(&conn, title)?;

    Ok(())
}