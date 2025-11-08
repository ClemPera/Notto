use std::sync::Mutex;

use chrono::NaiveDateTime;
use serde::Serialize;
use tauri::State;
use tauri_plugin_log::log::debug;

use crate::AppState;
use crate::crypt::NoteData;
use crate::db::operations::{self};
use crate::db::schema::{Note, User};

///Convert any error to string for frontend
#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: u32,
    pub username: String,
}

impl From<User> for FilteredUser {
    fn from(user: User) -> Self{
        FilteredUser {
            id: user.id.unwrap(),
            username: user.username
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NoteMetadata {
    pub id: u32,
    pub title: String,
    pub created_at: NaiveDateTime,
}

impl From<Note> for NoteMetadata {
    fn from(note: Note) -> Self {
        NoteMetadata {
            id: note.id.unwrap(),
            title: note.title,
            created_at: note.created_at.unwrap()
        }
    }
}

impl From<Box<dyn std::error::Error>> for CommandError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

#[tauri::command]
pub fn init(state: State<'_, Mutex<AppState>>) {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();

    operations::init(&conn);
}

#[tauri::command]
pub fn create_note(state: State<'_, Mutex<AppState>>, title: String) -> Result<(), CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    operations::create_note(&conn, state.id_user.unwrap(), title, state.master_encryption_key.unwrap()).unwrap();

    Ok(())
}

#[tauri::command]
pub fn get_note(state: State<'_, Mutex<AppState>>, id: u32) -> Result<NoteData, CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    let note = operations::get_note(&conn, id, state.master_encryption_key.unwrap()).unwrap();

    Ok(note)
}

#[tauri::command]
pub fn edit_note(state: State<'_, Mutex<AppState>>, note: NoteData) -> Result<(), CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    

    operations::update_note(&conn, note, state.master_encryption_key.unwrap()).unwrap();

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_all_notes_metadata(state: State<'_, Mutex<AppState>>, id_user: u32) -> Result<Vec<NoteMetadata>, CommandError> {    
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();

    let notes = operations::get_notes(&conn, id_user).unwrap();

    let notes_metadata = notes.into_iter().map(NoteMetadata::from).collect();
    
    Ok(notes_metadata)
}

#[tauri::command]
pub fn create_user(state: State<'_, Mutex<AppState>>, username: String, password: String) -> Result<(), CommandError> {
    let mut state = state.lock().unwrap();

    let user = {
        let conn = state.database.lock().unwrap();
        operations::create_user(&conn, username, password).unwrap()
    };

    state.master_encryption_key = Some(user.master_encryption_key);
    state.id_user = user.id;

    debug!("user created");
    
    Ok(())
}

#[tauri::command]
pub fn get_users(state: State<'_, Mutex<AppState>>) -> Result<Vec<FilteredUser>, CommandError> {
    let state = state.lock().unwrap();

    let conn = state.database.lock().unwrap();
    
    let users = operations::get_users(&conn).unwrap();

    let filtered_users= users.into_iter().map(FilteredUser::from).collect();

    Ok(filtered_users)
}

#[tauri::command]
pub fn test(state: State<'_, Mutex<AppState>>) -> Result<(), CommandError> {
    let state = state.lock().unwrap();
    
    debug!("mek is: {:?}", state.master_encryption_key);

    Ok(())
}

#[tauri::command]
pub fn set_user(state: State<'_, Mutex<AppState>>, id: u32) -> Result<(), CommandError> {
    let mut state = state.lock().unwrap();
    
    let user = {
        let conn = state.database.lock().unwrap();
        operations::get_user(&conn, id).unwrap()
    };

    state.id_user = Some(id);

    state.master_encryption_key = Some(user.master_encryption_key);

    Ok(())
}