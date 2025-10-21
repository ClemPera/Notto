use crate::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateFolderResponse {
    pub folder_id: String,
}

#[derive(Debug, Serialize)]
pub struct ListFoldersResponse {
    pub folder_ids: Vec<String>,
}

/// Create a new folder
#[tauri::command]
pub async fn create_folder(
    state: tauri::State<'_, AppState>,
    req: CreateFolderRequest,
) -> Result<CreateFolderResponse, String> {
    let db = state.db.lock().map_err(|e| format!("Database lock failed: {}", e))?;
    let folder_id = crate::db::operations::create_folder(
        &db,
        "default_user", // TODO: Get from session
        &req.name,
        req.parent_id.as_deref(),
    ).map_err(|e| format!("Failed to create folder: {}", e))?;

    Ok(CreateFolderResponse { folder_id })
}

/// List all folders
#[tauri::command]
pub async fn list_folders(
    state: tauri::State<'_, AppState>,
) -> Result<ListFoldersResponse, String> {
    let db = state.db.lock().map_err(|e| format!("Database lock failed: {}", e))?;
    let folder_ids = crate::db::operations::list_folders(&db, "default_user")
        .map_err(|e| format!("Failed to list folders: {}", e))?;

    Ok(ListFoldersResponse { folder_ids })
}

