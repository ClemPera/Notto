use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));
        // `POST /users` goes to `create_user`
        // .route("/users", post(create_user));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}