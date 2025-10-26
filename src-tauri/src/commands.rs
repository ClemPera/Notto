use tauri::State;

use crate::AppState;

use crate::db::operations;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn create_note(state: State<'_, AppState>) {
    let conn = state.database.lock().unwrap();
    
    operations::create_note(&conn);
}