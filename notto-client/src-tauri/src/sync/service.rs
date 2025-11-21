use std::{thread, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::debug;

use crate::AppState;

pub async fn run(handle: AppHandle) {
    let state = handle.state::<Mutex<AppState>>();
    let mut last_sync = DateTime::<Utc>::MIN_UTC.naive_utc();

    loop{
        debug!("Hello, I'm a background service! Here's the current user_id: {:?}", state.lock().await.id_user);

        //TODO: ask server to send back what has been modified since last_sync.
        //          -> server return a list of note
        //      Client send back the list of what it want to update
        //          -> server send a Vec<Note> for what the client asked.
        //TODO: modify last_sync with latest utc datetime 

        crate::sync::sync(&state.lock().await.database).await;
        
        thread::sleep(Duration::from_secs(1));
    }
}