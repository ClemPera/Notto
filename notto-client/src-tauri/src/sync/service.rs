use std::{thread, time::Duration};

use tokio::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::debug;

use crate::AppState;

pub async fn run(handle: AppHandle) {
    loop{
        let state = handle.state::<Mutex<AppState>>();
    
        debug!("Hello, I'm a background service! Here's the current user_id: {:?}", state.lock().await.id_user);
        thread::sleep(Duration::from_secs(1));
    }
}