use std::env;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post, put},
};
use dotenv::dotenv;
use mysql_async::{Conn, Pool};
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};

mod schema;

#[derive(Deserialize)]
struct NoteParams {
    id_user: u32,
    token: Vec<u8>
}

#[derive(Deserialize)]
struct Note {
    note: schema::Note,
    token: Vec<u8>
}

#[derive(Deserialize)]
struct UserParams {
    id_user: u32,
}

#[derive(Serialize)]
struct LoginRequest {
    salt_auth: String,
    salt_server_auth: String,
}

#[derive(Deserialize)]
struct Login {
    id_user: u32,
    login_hash: String,
}

#[derive(Serialize)]
struct LoginResponse {
    salt_data: String,
    encrypted_mek_password: Vec<u8>,
    token: Vec<u8>,
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
        // .route("/user", put()) //Update user
        .route("/login", get(login_request)) //Request login
        .route("/login", post(login)) //Check login hash
        // .route("/user_recovery", get()) //Request recovery stuff
        // .route("/user_recovery", post()) //check recovery hash
        // .route("/data_recovery", get()) //Request recovery stuff
        // .route("/data_recovery", post()) //store new recovery stuff
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn user_verify(conn: &mut Conn , id: u32, token: Vec<u8>) -> Result<(), StatusCode> {
    let user_token = schema::UserToken::select(conn, id).await;

    if user_token.token == token {
        Ok(())
    }else{
        Err(StatusCode::FORBIDDEN)
    }
}

async fn insert_note(State(pool): State<Pool>, Json(note): Json<Note>) -> Result<StatusCode, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();

    user_verify(&mut conn, note.note.id_user, note.token).await?;

    note.note.insert(&mut conn).await;

    Ok(StatusCode::OK)
}

async fn update_note(State(pool): State<Pool>, Json(note): Json<Note>) -> Result<StatusCode, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();

    user_verify(&mut conn, note.note.id_user, note.token).await?;

    note.note.update(&mut conn).await;

    Ok(StatusCode::OK)
}

async fn select_notes(
    State(pool): State<Pool>,
    Query(params): Query<NoteParams>,
) -> Result<Json<Vec<schema::Note>>, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();
    user_verify(&mut conn, params.id_user, params.token).await?;

    let notes = schema::Note::select_all_from_user(&mut conn, params.id_user).await;

    Ok(Json(notes))
}

async fn insert_user(State(pool): State<Pool>, Json(user): Json<schema::User>) {
    let mut conn = pool.get_conn().await.unwrap();

    user.insert(&mut conn).await;
}

async fn login_request(
    State(pool): State<Pool>,
    Query(param): Query<UserParams>,
) -> Json<LoginRequest> {
    let mut conn = pool.get_conn().await.unwrap();

    let user = schema::User::select(&mut conn, param.id_user).await;

    Json(LoginRequest {
        salt_auth: user.salt_auth,
        salt_server_auth: user.salt_server_auth,
    })
}

#[axum::debug_handler]
async fn login(
    State(pool): State<Pool>,
    Json(param): Json<Login>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();

    //Check if login_hash is correct
    let user = schema::User::select(&mut conn, param.id_user).await;

    if param.login_hash != user.stored_password_hash {
        return Err(StatusCode::UNAUTHORIZED);
    }

    //Generate token
    let mut token = vec![0u8; 32];
    OsRng.try_fill_bytes(&mut token).unwrap();


    //Store token
    let user_token = schema::UserToken {
        id: None,
        id_user: param.id_user,
        token,
    };

    user_token.insert(&mut conn).await;

    //Response
    Ok(Json(LoginResponse {
        salt_data: user.salt_data,
        encrypted_mek_password: user.encrypted_mek_password,
        token: user_token.token,
    }))
}
