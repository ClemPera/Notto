use std::env;

use axum::{extract::State, http::StatusCode, routing::{get, post, put}, Router};
use mysql_async::{Conn, Pool};

#[tokio::main]
async fn main() {
    let pool = Pool::new(env::var("DATABASE_URL").unwrap().as_str());
    
    let app = Router::new()
        .route("/note", post(create_note))
        .route("/note", put(update_note))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
// async fn root() -> &'static str {
//     "Hello, World!"
// }

async fn create_note(State(pool): State<Pool>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();

    StatusCode::NOT_IMPLEMENTED
}

async fn update_note(State(pool): State<Pool>) -> StatusCode{
    let mut conn = pool.get_conn().await.unwrap();

    StatusCode::NOT_IMPLEMENTED
}