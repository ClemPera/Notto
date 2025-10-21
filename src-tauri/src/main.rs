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
            // Note commands
            commands::notes::create_note,
            commands::notes::get_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::list_notes,
            // Folder commands
            commands::folders::create_folder,
            commands::folders::list_folders,
            // Auth commands
            commands::auth::register,
            commands::auth::login,
            commands::auth::setup_totp,
            commands::auth::verify_totp_setup,
            commands::auth::verify_session_token,
            commands::auth::logout,
            // Sync commands
            commands::sync::initialize_sync,
            commands::sync::start_sync,
            commands::sync::get_sync_status,
            commands::sync::check_connectivity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

