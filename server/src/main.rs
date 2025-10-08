use axum::{
    Router,
    routing::{get, post},
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;

mod user_operations;
use user_operations::{change_password, create_user, delete_user, login_user};

mod message_operations;

#[tokio::main]
async fn main() {
    let pool = connect_to_database().await;
    let shared_state = Arc::new(pool);

    println!("Starting the http server...");

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/status", get(|| async { "Status: OK" }))
        .route("/create_user", post(create_user))
        .route("/change_password", post(change_password))
        .route("/login", post(login_user))
        .route("/delete_user", post(delete_user))
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
