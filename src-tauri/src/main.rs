// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod db;
mod crypto;
mod models;
mod commands;
mod sync;
mod auth;

use db::DbConnection;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub db: DbConnection,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    // Initialize database
    let app_dir = tauri::api::path::app_data_dir(&tauri::Config::default()).unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    });

    let db_path = app_dir.join("notto.db");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_dir).unwrap_or_else(|_| {
        eprintln!("Warning: Could not create app directory");
    });

    let db = db::init(&db_path).expect("Failed to initialize database");

    let app_state = AppState { db };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::notes::create_note,
            commands::notes::get_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::list_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
