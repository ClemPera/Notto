use rusqlite::Connection;
use crate::schema::User;

mod operations;

pub fn create_account(conn: &Connection, user: User, instance: Option<String>){
    let instance = match instance {
        Some(i) => i,
        None => "localhost:3000".to_string()
    };

    //TODO
    // operations::create_account(user.into(), instance);
}