use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

pub async fn connect_to_database() -> Pool<Postgres> {
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

    println!("Connected to the database.");
    pool
}

pub async fn create_user(
    State(pool): State<Arc<Pool<Postgres>>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ApiResponse> {
    let password_hash = hash_password(&payload.password);

    let query_result =
        sqlx::query("INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3)")
            .bind(payload.username)
            .bind(payload.email)
            .bind(password_hash)
            .execute(&*pool)
            .await;

    match query_result {
        Ok(_) => Json(ApiResponse {
            status: "success".to_string(),
            message: "User created successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Failed to create user: {}", e),
            })
        }
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
