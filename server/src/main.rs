use axum::{
    Router,
    routing::{get, post},
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

mod user_operations;
use user_operations::{change_password, create_user, delete_user, login_user};

mod websocket_handler;
use websocket_handler::{Tx, websocket_handler};

#[tokio::main]
async fn main() {
    let pool = connect_to_database().await;

    // broadcast channel for WebSocket messages
    let (tx, _rx): (Tx, _) = broadcast::channel(100);

    let shared_state = Arc::new((pool, tx));

    println!("Starting the http server...");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/status", get(|| async { "Status: OK" }))
        .route("/create_user", post(create_user))
        .route("/change_password", post(change_password))
        .route("/login", post(login_user))
        .route("/delete_user", post(delete_user))
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running at http://{}", addr);
    println!("WebSocket endpoint available at ws://{}/ws", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

async fn connect_to_database() -> Pool<Postgres> {
    println!("Connecting to the database...");

    let db_url = "postgres://arjav:arjav@localhost/user_db";
    dbg!(&db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to create pool.");

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) NOT NULL UNIQUE,
            password_hash VARCHAR(256) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages(
            id SERIAL PRIMARY KEY,
            user_id INT REFERENCES users(id),
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await;

    println!("Connected to the database.");
    pool
}
