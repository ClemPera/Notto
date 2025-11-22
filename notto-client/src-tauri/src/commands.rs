use tokio::sync::Mutex;

use chrono::NaiveDateTime;
use serde::Serialize;
use tauri::State;
use tauri_plugin_log::log::{debug, trace};

use crate::{AppState, crypt, sync};
use crate::crypt::NoteData;
use crate::db;
use crate::db::schema::{Note, User};

///Convert any error to string for frontend
#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

impl From<Box<dyn std::error::Error>> for CommandError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
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
    pub updated_at: NaiveDateTime,
}

impl From<Note> for NoteMetadata {
    fn from(note: Note) -> Self {
        NoteMetadata {
            id: note.id.unwrap(),
            title: note.title,
            updated_at: note.updated_at
        }
    }
}

#[tauri::command]
pub async fn init(state: State<'_, Mutex<AppState>>) -> Result<(), CommandError>  {
    let state = state.lock().await;

    let conn = state.database.lock().await;

    db::operations::init(&conn);
    
    Ok(())
}

#[tauri::command]
pub async fn create_note(state: State<'_, Mutex<AppState>>, title: String) -> Result<(), CommandError> {
    let state = state.lock().await;

    let conn = state.database.lock().await;
    
    db::operations::create_note(&conn, state.id_user.unwrap(), title, state.master_encryption_key.unwrap()).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn get_note(state: State<'_, Mutex<AppState>>, id: u32) -> Result<NoteData, CommandError> {
    let state = state.lock().await;

    let conn = state.database.lock().await;
    
    let note = db::operations::get_note(&conn, id, state.master_encryption_key.unwrap()).unwrap();

    Ok(note)
}

#[tauri::command]
pub async fn edit_note(state: State<'_, Mutex<AppState>>, note: NoteData) -> Result<(), CommandError> {
    let state = state.lock().await;

    let conn = state.database.lock().await;

    db::operations::update_note(&conn, note, state.master_encryption_key.unwrap()).unwrap();

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_notes_metadata(state: State<'_, Mutex<AppState>>, id_user: u32) -> Result<Vec<NoteMetadata>, CommandError> {    
    let state = state.lock().await;

    let conn = state.database.lock().await;

    let notes = db::operations::get_notes(&conn, id_user).unwrap();

    let notes_metadata = notes.into_iter().map(NoteMetadata::from).collect();
    
    Ok(notes_metadata)
}

#[tauri::command]
pub async fn create_user(state: State<'_, Mutex<AppState>>, username: String) -> Result<(), CommandError> {
    let mut state = state.lock().await;

    let user = {
        let conn = state.database.lock().await;
        db::operations::create_user(&conn, username).unwrap()
    };

    state.master_encryption_key = Some(user.master_encryption_key);
    state.id_user = user.id;

    debug!("user created");
    
    Ok(())
}

#[tauri::command]
pub async fn get_users(state: State<'_, Mutex<AppState>>) -> Result<Vec<FilteredUser>, CommandError> {
    let state = state.lock().await;

    let conn = state.database.lock().await;
    
    let users = db::operations::get_users(&conn).unwrap();

    let filtered_users= users.into_iter().map(FilteredUser::from).collect();

    Ok(filtered_users)
}

#[tauri::command]
pub async fn test(state: State<'_, Mutex<AppState>>) -> Result<(), CommandError> {
    let state = state.lock().await;
    
    debug!("mek is: {:?}", state.master_encryption_key);

    Ok(())
}

#[tauri::command]
pub async fn set_user(state: State<'_, Mutex<AppState>>, username: String) -> Result<(), CommandError> {
    let mut state = state.lock().await;
    
    let user = {
        let conn = state.database.lock().await;
        match db::operations::get_user(&conn, username).unwrap() {
            Some(u) => u,
            None => return Err(CommandError { message: "User doesn't exist".to_string() })
        }
    };

    state.master_encryption_key = Some(user.master_encryption_key);
    state.id_user = Some(user.id.unwrap());
    state.token = user.token;
    state.instance = user.instance;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_create_account(state: State<'_, Mutex<AppState>>, username: String, password: String, instance: Option<String>) -> Result<(), CommandError> {
    trace!("create account command received");
    
    let mut state = state.lock().await;
    
    let conn = state.database.lock().await;
    let user = db::operations::get_user(&conn, username).unwrap().unwrap();
    let account = crypt::create_account(password, state.master_encryption_key.unwrap());
    
    trace!("create account: start creating");
    sync::create_account(&conn, user, account, instance);
    
    debug!("account has been created");

    //TODO: send back recovery key to frontend
    
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_login(state: State<'_, Mutex<AppState>>, username: String, password: String, instance: Option<String>) -> Result<(), CommandError> {
    trace!("create account command received");

    let mut state = state.lock().await;

    let login_data = {
        let conn = state.database.lock().await;
        sync::login(&conn, username.clone(), password.clone(), instance.clone())
    };
    debug!("account has been logged in");

    let mut user = {
        let conn = state.database.lock().await;
        match db::operations::get_user(&conn, username).unwrap() {
            Some(u) => u,
            None => return Err(CommandError { message: "User doesn't exist".to_string() })
        }
    };

    //TODO: if !user.has_mek() then do not decrypt mek?
    //TODO: handle if user account not created locally?

    let mek = crypt::decrypt_mek(password, login_data.encrypted_mek_password, login_data.salt_data, login_data.mek_password_nonce);
    state.master_encryption_key = Some(mek);
    state.token = Some(login_data.token.clone());
    state.instance = instance;

    user.master_encryption_key = mek;
    user.id_server = Some(login_data.id_server);
    user.token = Some(login_data.token);

    {
        let conn = state.database.lock().await;
        db::operations::update_user(&conn, user);
    }

    Ok(())
}