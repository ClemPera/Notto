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

mod schema;

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

async fn insert_note(State(pool): State<Pool>, Json(sent_note): Json<shared::SentNote>) -> Result<StatusCode, StatusCode> {
    let note: schema::Note = sent_note.note.into();
    let mut conn = pool.get_conn().await.unwrap();

    user_verify(&mut conn, note.id_user, sent_note.token).await?;

    note.insert(&mut conn).await;

    Ok(StatusCode::OK)
}

async fn update_note(State(pool): State<Pool>, Json(sent_note): Json<shared::SentNote>) -> Result<StatusCode, StatusCode> {
    let note: schema::Note = sent_note.note.into();
    let mut conn = pool.get_conn().await.unwrap();

    user_verify(&mut conn, note.id_user, sent_note.token).await?;

    note.update(&mut conn).await;

    Ok(StatusCode::OK)
}

async fn select_notes(
    State(pool): State<Pool>,
    Query(params): Query<shared::SelectNoteParams>,
) -> Result<Json<Vec<schema::Note>>, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();
    user_verify(&mut conn, params.id_user, params.token).await?;

    let notes = schema::Note::select_all_from_user(&mut conn, params.id_user).await;

    Ok(Json(notes))
}

async fn insert_user(State(pool): State<Pool>, Json(user): Json<shared::User>) {
    println!("received insert_user");
    let user: schema::User = user.into();
    
    let mut conn = pool.get_conn().await.unwrap();
    
    user.insert(&mut conn).await;
    println!("insert_user: completed");
}

async fn login_request(
    State(pool): State<Pool>,
    Query(params): Query<shared::LoginRequestParams>,
) -> Json<shared::LoginRequest> {
    let mut conn = pool.get_conn().await.unwrap();

    let user = schema::User::select(&mut conn, params.username).await;
    
    Json(shared::LoginRequest {
        salt_auth: user.salt_auth,
        salt_server_auth: user.salt_server_auth,
    })
}

#[axum::debug_handler]
async fn login(
    State(pool): State<Pool>,
    Json(params): Json<shared::LoginParams>,
) -> Result<Json<shared::Login>, StatusCode> {
    let mut conn = pool.get_conn().await.unwrap();

    //Check if login_hash is correct
    let user = schema::User::select(&mut conn, params.username).await;

    if params.login_hash != user.stored_password_hash {
        return Err(StatusCode::UNAUTHORIZED);
    }

    //Generate token
    let mut token = vec![0u8; 32];
    OsRng.try_fill_bytes(&mut token).unwrap();

    //Store token
    let user_token = schema::UserToken {
        id: None,
        id_user: user.id.unwrap(),
        token,
    };

    user_token.insert(&mut conn).await;

    //Response
    Ok(Json(shared::Login {
        salt_data: user.salt_data,
        encrypted_mek_password: user.encrypted_mek_password,
        mek_password_nonce: user.mek_password_nonce,
        token: user_token.token,
    }))
}
