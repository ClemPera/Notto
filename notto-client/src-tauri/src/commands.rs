use std::sync::Mutex;

use serde::Serialize;
use tauri::State;
use tauri_plugin_log::log::debug;

use crate::AppState;
use crate::db::operations::{self, FilteredUser};

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
pub fn init(state: State<'_, Mutex<AppState>>) {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();

    operations::init(&conn);
}

#[tauri::command]
pub fn create_note(state: State<'_, Mutex<AppState>>, title: String) -> Result<(), CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    operations::create_note(&conn, title, state.master_encryption_key.unwrap()).unwrap();

    Ok(())
}

#[tauri::command]
pub fn get_note(state: State<'_, Mutex<AppState>>, id: u32) -> Result<String, CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    let note = operations::get_note(&conn, id, state.master_encryption_key.unwrap()).unwrap();

    Ok(note.title)
}

#[tauri::command]
pub fn create_account(state: State<'_, Mutex<AppState>>, username: String, password: String) -> Result<(), CommandError> {
    let mut state = state.lock().unwrap();

    let user = {
        let conn = state.database.lock().unwrap();
        operations::create_account(&conn, username, password).unwrap()
    };

    state.master_encryption_key = Some(user.master_encryption_key);

    debug!("account created");
    
    Ok(())
}

#[tauri::command]
pub fn get_users(state: State<'_, Mutex<AppState>>) -> Result<Vec<FilteredUser>, CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    let users = operations::get_users(&conn).unwrap();

    Ok(users)
}

#[tauri::command]
pub fn test(state: State<'_, Mutex<AppState>>) -> Result<(), CommandError> {
    let state = state.lock().unwrap();
    
    debug!("mek is: {:?}", state.master_encryption_key);

    Ok(())
}