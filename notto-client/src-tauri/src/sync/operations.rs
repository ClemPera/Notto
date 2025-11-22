use shared::{LoginRequestParams, Note, SelectNoteParams, SentNote, User};
use tauri_plugin_log::log::{trace, debug};

pub fn send_note(note: SentNote, instance: String) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let response = client.post(instance + "/note").json(&note).send().unwrap().error_for_status()?;

    return Ok(response.json().unwrap())
}

pub fn select_notes(params :SelectNoteParams, instance: String) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let response = client.get(instance + "/note").query(&params).send().unwrap().error_for_status()?;
    //TODO: handle StatusCode::Conflict at some point

    Ok(response.json().unwrap())
}

pub fn create_account(user: User, instance: String){
    let client = reqwest::blocking::Client::new();
    let response = client.post(instance + "/create_account").json(&user).send().unwrap();

    trace!("create account response: {response:?}");
}

pub fn login_request(params: LoginRequestParams, instance: String) -> shared::LoginRequest{
    let client = reqwest::blocking::Client::new();

    client.get(instance + "/login").query(&params).send().unwrap().json().unwrap()
}

pub fn login(params: shared::LoginParams, instance: String) -> Result<shared::Login, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let response = client.post(instance + "/login").json(&params).send().unwrap().error_for_status()?;

    return Ok(response.json().unwrap())
}