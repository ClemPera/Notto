use std::env;

use axum::{Json, Router, extract::{Query, State}, http::StatusCode, routing::{get, post, put}};
use dotenv::dotenv;
use mysql_async::{Pool};
use serde::Deserialize;

use crate::schema::Note;

mod schema;

#[derive(Deserialize)]
struct GetNoteParams {
    id_user: u32,
}


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
    
    let mut conn = pool.get_conn().await.unwrap();

    note.create(&mut conn).await;

    StatusCode::OK
}

async fn update_note(State(pool): State<Pool>, Json(note): Json<Note>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();
    
    note.update(&mut conn).await;

    StatusCode::OK
}

async fn get_note(State(pool): State<Pool>, Query(params): Query<GetNoteParams>) -> Json<Vec<Note>>{
    let mut conn = pool.get_conn().await.unwrap();

    let notes = Note::select_all(&mut conn, params.id_user).await;

    Json(notes)
}