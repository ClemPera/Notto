use std::env;

use axum::{extract::State, http::StatusCode, routing::{get, post, put}, Json, Router};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use mysql_async::{Conn, Pool};

use crate::schema::Note;

mod schema;

#[tokio::main]
async fn main() {
    dotenv().ok();
    //Env var should be like mysql://user:pass%20word@localhost/database_name
    let pool = Pool::new(env::var("DATABASE_URL").unwrap().as_str());
    
    let app = Router::new()
        .route("/note", post(create_note))
        .route("/note", put(update_note))
        .route("/note", get(get_note))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
// async fn root() -> &'static str {
//     "Hello, World!"
// }

async fn create_note(State(pool): State<Pool>, Json(note): Json<Note>) -> StatusCode{
    //TODO: add user verif
    
    let conn = pool.get_conn().await.unwrap();

    //TODO
    note.create(&conn);

    StatusCode::NOT_IMPLEMENTED
}

async fn update_note(State(pool): State<Pool>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();

    StatusCode::NOT_IMPLEMENTED
}

async fn get_note(State(pool): State<Pool>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();

    StatusCode::NOT_IMPLEMENTED
}