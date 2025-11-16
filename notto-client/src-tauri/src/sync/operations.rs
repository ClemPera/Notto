use shared::User;
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

pub fn login_request(){

}

pub fn login(){

}