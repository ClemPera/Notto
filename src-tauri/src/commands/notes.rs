use crate::AppState;
use serde::Deserialize;
use tauri_plugin_log::log::{info, warn};

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub folder_id: Option<String>,
}

#[tauri::command]
pub async fn create_note(
    state: tauri::State<'_, AppState>,
    req: CreateNoteRequest,
) -> Result<String, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock failed: {}", e))?;
    let note_id = crate::db::operations::create_note(
        &db,
        "61da3ff8-e06e-4ab2-99e0-8c258cc33354", // TODO: Get from session
        &req.title,
        req.content.as_bytes(),
        req.folder_id.as_deref(),
    )
    .map_err(|e| format!("Failed to create note: {}", e))?;

    Ok(note_id)
}

#[tauri::command]
pub async fn get_note(
    state: tauri::State<'_, AppState>,
    note_id: String,
) -> Result<String, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock failed: {}", e))?;
    let content = crate::db::operations::get_note(&db, &note_id)
        .map_err(|e| format!("Failed to get note: {}", e))?
        .ok_or("Note not found")?;

    let content_str = String::from_utf8(content).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    Ok(content_str)
}

#[tauri::command]
pub async fn update_note(
    state: tauri::State<'_, AppState>,
    note_id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    warn!("noteid2:{note_id:?}");
    
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock failed: {}", e))?;
    
    crate::db::operations::update_note(&db, &note_id, &title, content.as_bytes())
        .map_err(|e| format!("Failed to update note: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn delete_note(state: tauri::State<'_, AppState>, note_id: String) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock failed: {}", e))?;
    crate::db::operations::delete_note(&db, &note_id)
        .map_err(|e| format!("Failed to delete note: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn list_notes(
    state: tauri::State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock failed: {}", e))?;
    let note_ids = crate::db::operations::list_notes(&db, "default_user", folder_id.as_deref())
        .map_err(|e| format!("Failed to list notes: {}", e))?;
    Ok(note_ids)
}
