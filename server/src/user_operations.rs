use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
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

async fn check_user_exists(pool: &Pool<Postgres>, email: &str) -> bool {
    let result = sqlx::query("SELECT 1 FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            false
        }
    }
}

pub async fn create_user(
    State(state): State<Arc<(Pool<Postgres>, tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>)>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ApiResponse> {
    let pool = &state.0;
    let password_hash = hash_password(&payload.password);

    if check_user_exists(pool, &payload.email).await {
        return Json(ApiResponse {
            status: "error".to_string(),
            message: "User with this email already exists".to_string(),
        });
    }

    let query_result =
        sqlx::query("INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3)")
            .bind(payload.username)
            .bind(payload.email)
            .bind(password_hash)
            .execute(pool)
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

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub email: String,
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<Arc<(Pool<Postgres>, tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>)>>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Json<ApiResponse> {
    let pool = &state.0;
    if !authenticate_user(pool, &payload.email, &payload.old_password).await {
        return Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid email or password".to_string(),
        });
    }

    let password_hash = hash_password(&payload.new_password);

    let query_result = sqlx::query("UPDATE users SET password_hash = $1 WHERE email = $2")
        .bind(password_hash)
        .bind(payload.email)
        .execute(pool)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Json(ApiResponse {
                    status: "error".to_string(),
                    message: "No user found with this email".to_string(),
                })
            } else {
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Password changed successfully".to_string(),
                })
            }
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Failed to change password: {}", e),
            })
        }
    }
}

async fn authenticate_user(pool: &Pool<Postgres>, email: &str, password: &str) -> bool {
    let password_hash = hash_password(password);
    if !check_user_exists(pool, email).await {
        return false;
    }
    let result = sqlx::query("SELECT 1 FROM users WHERE email = $1 AND password_hash = $2")
        .bind(email)
        .bind(password_hash)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            false
        }
    }
}

pub async fn delete_user(
    State(state): State<Arc<(Pool<Postgres>, tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>)>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ApiResponse> {
    let pool = &state.0;
    if !authenticate_user(pool, &payload.email, &payload.password).await {
        return Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid email or password".to_string(),
        });
    }

    let query_result = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(payload.email)
        .execute(pool)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Json(ApiResponse {
                    status: "error".to_string(),
                    message: "No user found with this email".to_string(),
                })
            } else {
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "User deleted successfully".to_string(),
                })
            }
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Failed to delete user: {}", e),
            })
        }
    }
}

pub async fn login_user(
    State(state): State<Arc<(Pool<Postgres>, tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>)>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ApiResponse> {
    let pool = &state.0;
    if authenticate_user(pool, &payload.email, &payload.password).await {
        Json(ApiResponse {
            status: "success".to_string(),
            message: "Login successful".to_string(),
        })
    } else {
        Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid email or password".to_string(),
        })
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
