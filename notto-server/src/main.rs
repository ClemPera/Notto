use std::env;

use axum::{Json, Router, extract::{Query, State}, http::StatusCode, routing::{get, post, put}};
use dotenv::dotenv;
use mysql_async::{Pool};
use serde::Deserialize;

use crate::schema::{Note, User};

mod schema;

#[derive(Deserialize)]
struct NoteParams {
    id_user: u32,
}

#[derive(Deserialize)]
struct UserParams {
    id: u32,
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    //Env var should be like mysql://user:pass%20word@localhost/database_name
    let pool = Pool::new(env::var("DATABASE_URL").unwrap().as_str());
    
    let app = Router::new()
        .route("/note", post(insert_note))
        .route("/note", put(update_note))
        .route("/note", get(select_notes))

        .route("/create_account", post(insert_user)) //Create account

        .route("/user", put()) //Update user
        
        .route("/login", get()) //Request login
        .route("/login", post()) //Check login hash
        
        .route("/user_recovery", get()) //Request recovery stuff
        .route("/user_recovery", post()) //check recovery hash
        
        .route("/data_recovery", get()) //Request recovery stuff
        .route("/data_recovery", post()) //store new recovery stuff

        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
// async fn root() -> &'static str {
//     "Hello, World!"
// }

async fn insert_note(State(pool): State<Pool>, Json(note): Json<Note>) -> StatusCode{
    //TODO: add user verif
    
    let mut conn = pool.get_conn().await.unwrap();

    note.insert(&mut conn).await;

    StatusCode::OK
}

async fn update_note(State(pool): State<Pool>, Json(note): Json<Note>) -> StatusCode{
    //TODO: add user verif

    let mut conn = pool.get_conn().await.unwrap();
    
    note.update(&mut conn).await;

    StatusCode::OK
}

async fn select_notes(State(pool): State<Pool>, Query(params): Query<NoteParams>) -> Json<Vec<Note>>{
    //TODO: add user verif

    let mut conn = pool.get_conn().await.unwrap();

    let notes = Note::select_all_from_user(&mut conn, params.id_user).await;

    Json(notes)
}

async fn insert_user(State(pool): State<Pool>, Json(user): Json<User>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();

    user.insert(&mut conn).await;

    StatusCode::OK
}

async fn select_user(State(pool): State<Pool>, Query(params): Query<UserParams>) -> Json<User>{
    let mut conn = pool.get_conn().await.unwrap();

    let user = User::select(&mut conn, params.id).await;

    Json(user)
}