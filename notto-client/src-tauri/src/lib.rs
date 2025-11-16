// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command(rename_all = "snake_case")]

use std::sync::Mutex;

use aes_gcm::{Aes256Gcm, Key};
use rusqlite::Connection;
use tauri::Manager;

use crate::db::schema;

mod commands;
mod db;
mod crypt;
mod sync;

pub struct AppState {
  database: Mutex<Connection>,
  master_encryption_key: Option<Key<Aes256Gcm>>,
  id_user: Option<u32>
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Trace)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = app.path().app_data_dir().unwrap().join("notto.db");

            app.manage(Mutex::new(AppState{ 
                database: db::init(db_path).unwrap(),
                master_encryption_key: None,
                id_user: None,
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            commands::create_note,
            commands::get_note,
            commands::edit_note,
            commands::get_all_notes_metadata,
            commands::create_user,
            commands::get_users,
            commands::set_user,
            commands::sync_create_account,
            commands::test,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}