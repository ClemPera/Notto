use serde::Serialize;
use tauri::State;
use tauri_plugin_log::log::debug;

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
pub fn init(state: State<'_, AppState>) {
    let conn = state.database.lock().unwrap();

    operations::init(&conn);
}

#[tauri::command]
pub fn create_note(state: State<'_, AppState>, title: String) -> Result<(), CommandError> {
    let conn = state.database.lock().unwrap();
    
    operations::create_note(&conn, title).unwrap();

    Ok(())
}

#[tauri::command]
pub fn get_note(state: State<'_, AppState>, id: u32) -> Result<String, CommandError> {
    let conn = state.database.lock().unwrap();
    
    let note = operations::get_note(&conn, id).unwrap();

    Ok(note.title)
}

#[tauri::command]
pub fn create_account(state: State<'_, AppState>, username: String, password: String) -> Result<(), CommandError> {
    let conn = state.database.lock().unwrap();
    
    operations::create_account(&conn, username, password);
    debug!("account created");
    
    Ok(())
}