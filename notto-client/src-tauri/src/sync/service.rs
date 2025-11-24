use std::{thread, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use rusqlite::Connection;
use serde_json::error;
use shared::{SelectNoteParams, SentNotes};
use tokio::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::{debug, trace, error};

use crate::{AppState, db::{self, schema::Note}, sync};

pub async fn run(handle: AppHandle) {
    let state = handle.state::<Mutex<AppState>>();
    let mut last_sync = DateTime::<Utc>::MIN_UTC.naive_utc();

    loop{
        trace!("Hello, I'm a background service! Here's the current user_id: {:?}", state.lock().await.id_user);
        
        {
            let state = state.lock().await;

            if state.id_user.is_some() && state.token.is_some() && state.instance.is_some() {
                //Update sync infos
                let sync = Local::now().naive_utc();

                //Sync
                receive_latest_notes(&state, last_sync).await;
                send_latest_notes(&state).await;

                last_sync = sync;
            }else {
                debug!("Conditions are not respected to sync: {state:?}");
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}


pub async fn receive_latest_notes(state: &MutexGuard<'_, AppState>, last_sync: NaiveDateTime) {
    let conn = state.database.lock().await;

    let params = SelectNoteParams {
        id_user: state.id_user.clone().unwrap(),
        token: state.token.clone().unwrap(), 
        updated_at: last_sync
    };
    
    //Ask server for modified notes
    let notes = sync::operations::select_notes(params, state.instance.clone().unwrap()).unwrap();

    // Put new notes to database
    notes.into_iter().for_each(|note| {
        let note = db::schema::Note::from(note);
        
        //Check if exist
        let selected_note = db::schema::Note::select(&conn, note.id.unwrap()).unwrap();

        match selected_note {
            Some(sn) => {
                if note.updated_at > sn.updated_at {
                    //Note is more recent on server
                    match sn.synched {
                        true => note.update(&conn).unwrap(),
                        false => error!("Note {:?} is in conflict and it's not handled :(", sn.id) //TODO
                    };
                }
            },
            None => note.insert(&conn).unwrap()
        }

        //TODO: if deleted
    });
}

pub async fn send_latest_notes(state: &MutexGuard<'_, AppState>) {
    let conn = state.database.lock().await;
    
    //Fetch db find all notes with synched = false;
    let notes = Note::select_all(&conn, state.id_user.clone().unwrap()).unwrap();

    //TODO: Optimise that with a database query
    let notes: Vec<Note> = notes.into_iter().filter(|note| !note.synched).collect();

    let sent_notes = SentNotes {
        id_user: state.id_user.clone().unwrap(),
        notes: notes.into_iter().map(|n| n.into()).collect(),
        token: state.token.clone().unwrap()
    };

    //Send server these notes
    let results = sync::operations::send_notes(sent_notes, state.instance.clone().unwrap()).unwrap();

    //Handle Results
    results.into_iter().for_each(|result| {
        match result.status {
            shared::NoteStatus::Ok => {
                let mut note = Note::select(&conn, result.id_client).unwrap().unwrap();

                note.synched = true;
                note.id_server = Some(result.id_server);

                note.update(&conn).unwrap();
            },
            shared::NoteStatus::Conflict => {
                //TODO
                error!("Note {:?} is in conflict and it's not handled :(", result.id_client) 
            }
        }
    });
}