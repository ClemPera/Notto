use shared::{LoginRequestParams, User};
use tauri_plugin_log::log::debug;

pub fn insert_note(){

}

pub fn update_note(){

}

pub fn select_notes(){

}

pub fn create_account(user: User, instance: String){
    let client = reqwest::blocking::Client::new();
    let response = client.post(instance + "/create_account").json(&user).send().unwrap();

    debug!("create account response: {response:?}");
}

pub fn login_request(params: LoginRequestParams, instance: String) -> shared::LoginRequest{
    let client = reqwest::blocking::Client::new();

    client.get(instance + "/login").query(&params).send().unwrap().json().unwrap()
}

pub fn login(params: shared::LoginParams, instance: String) -> shared::Login{
    let client = reqwest::blocking::Client::new();

    client.post(instance + "/login").json(&params).send().unwrap().json().unwrap()
}