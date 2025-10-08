use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
mod user_operations;
use user_operations::{connect_to_database, create_user};

#[tokio::main]
async fn main() {
    let pool = connect_to_database().await;
    let shared_state = Arc::new(pool);

    println!("Starting the http server...");

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/status", get(|| async { "Status: OK" }))
        .route("/create_user", post(create_user))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}
