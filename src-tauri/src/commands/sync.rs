use crate::sync::{SyncClient, SyncStatus};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Global sync client (would normally be managed better)
lazy_static::lazy_static! {
    static ref SYNC_CLIENT: Mutex<Option<Arc<SyncClient>>> = Mutex::new(None);
}

#[derive(Debug, Deserialize)]
pub struct InitiateSyncRequest {
    pub token: String,
    pub server_url: String,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncProgressResponse {
    pub notes_synced: usize,
    pub total_notes: usize,
    pub percent_complete: u32,
}

/// Initialize sync client
#[tauri::command]
pub async fn initialize_sync(
    state: tauri::State<'_, AppState>,
    req: InitiateSyncRequest,
) -> Result<String, String> {
    // Verify session token
    let user_id = crate::auth::verify_session(&state.db, &req.token).map_err(|e| e.to_string())?;

    // Create sync client
    let sync_client = Arc::new(SyncClient::new(
        req.server_url,
        user_id.clone(),
        req.token,
        state.db.clone(),
    ));

    // Store in global state
    let mut client = SYNC_CLIENT.lock().await;
    *client = Some(sync_client);

    Ok("Sync initialized".to_string())
}

/// Start synchronization
#[tauri::command]
pub async fn start_sync() -> Result<SyncStatusResponse, String> {
    let client = SYNC_CLIENT.lock().await;

    if let Some(sync_client) = client.as_ref() {
        // Check connectivity first
        match sync_client.check_connectivity().await {
            Ok(true) => {
                // Start sync in background (fire and forget)
                let sync_client_clone = sync_client.clone();
                tokio::spawn(async move {
                    let _ = sync_client_clone.sync_all().await;
                });

                Ok(SyncStatusResponse {
                    status: "syncing".to_string(),
                    message: Some("Sync started".to_string()),
                })
            }
            Ok(false) => Ok(SyncStatusResponse {
                status: "offline".to_string(),
                message: Some("Cannot reach sync server".to_string()),
            }),
            Err(e) => Err(format!("Connectivity check failed: {}", e)),
        }
    } else {
        Err("Sync not initialized".to_string())
    }
}

/// Get current sync status
#[tauri::command]
pub async fn get_sync_status() -> Result<SyncStatusResponse, String> {
    let client = SYNC_CLIENT.lock().await;

    if let Some(sync_client) = client.as_ref() {
        let status = sync_client.get_status().await;

        let (status_str, message) = match status {
            SyncStatus::Idle => ("idle".to_string(), None),
            SyncStatus::Syncing => ("syncing".to_string(), Some("Sync in progress".to_string())),
            SyncStatus::Success => (
                "success".to_string(),
                Some("Last sync successful".to_string()),
            ),
            SyncStatus::Error(e) => ("error".to_string(), Some(e)),
            SyncStatus::Offline => ("offline".to_string(), Some("Offline mode".to_string())),
        };

        Ok(SyncStatusResponse {
            status: status_str,
            message,
        })
    } else {
        Ok(SyncStatusResponse {
            status: "not_initialized".to_string(),
            message: Some("Sync not initialized".to_string()),
        })
    }
}

/// Check if server is reachable
#[tauri::command]
pub async fn check_connectivity(server_url: String) -> Result<bool, String> {
    crate::sync::client::check_server_connectivity(&server_url)
        .await
        .map_err(|e| e.to_string())
}
