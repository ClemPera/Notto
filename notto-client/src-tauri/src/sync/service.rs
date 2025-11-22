use std::{thread, time::Duration};

use chrono::{DateTime, Local, Utc};
use shared::SelectNoteParams;
use tokio::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::debug;

use crate::{AppState, sync, db};

pub async fn run(handle: AppHandle) {
    let state = handle.state::<Mutex<AppState>>();
    let mut last_sync = DateTime::<Utc>::MIN_UTC.naive_utc();

    loop{
        debug!("Hello, I'm a background service! Here's the current user_id: {:?}", state.lock().await.id_user);
        
        {
            let state = state.lock().await;
            let conn = state.database.lock().await;

            if state.id_user.is_some() && state.token.is_some() && state.instance.is_some() {
                let params = SelectNoteParams {
                    id_user: state.id_user.unwrap(),
                    token: state.token.clone().unwrap(), 
                    updated_at: last_sync
                };
    
                //Update sync infos
                last_sync = Local::now().naive_utc();

                //Ask server for modified notes
                let notes = sync::operations::select_notes(params, state.instance.clone().unwrap()).unwrap();

                // Put new notes to database
                notes.into_iter().for_each(|note| {
                    // let note = db::schema::Note::from(note);
                    
                    //Check if exist
                    // let selected_note = db::schema::Note::select(&conn, note.id.unwrap()).unwrap();

                    // match selected_note {
                    //     Some(_) => 
                    //     None => 
                    // }


                    //TODO: if deleted
                });

            }else {
                debug!("Conditions are not respected to sync: {state:?}");
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}