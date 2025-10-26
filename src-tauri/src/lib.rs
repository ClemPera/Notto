// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command(rename_all = "snake_case")]

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

mod commands;
mod db;
mod crypt;

pub struct AppState {
  database: Mutex<Connection>,
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

            app.manage(AppState{ 
                database: db::init(db_path).unwrap() 
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::create_note
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}