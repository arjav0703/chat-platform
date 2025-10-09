use axum::extract::{Json, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: i32,
    pub user_email: String,
    pub username: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub status: String,
    pub messages: Vec<MessageResponse>,
}

pub async fn get_messages(
    State(state): State<Arc<(Pool<Postgres>, tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>)>>,
    Query(params): Query<GetMessagesQuery>,
) -> Json<MessagesResponse> {
    let pool = &state.0;
    let limit = params.limit.unwrap_or(100).min(500); // Default 100, max 500

    let query_result = sqlx::query_as::<_, (i32, String, String, String, String)>(
        "SELECT m.id, u.email, u.name, m.content, m.created_at::text 
         FROM messages m
         JOIN users u ON m.user_id = u.id
         ORDER BY m.created_at DESC
         LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await;

    match query_result {
        Ok(rows) => {
            let mut messages: Vec<MessageResponse> = rows
                .into_iter()
                .map(|(id, email, username, content, timestamp)| MessageResponse {
                    id,
                    user_email: email,
                    username,
                    content,
                    timestamp,
                })
                .collect();
            
            // Reverse to get chronological order (oldest first)
            messages.reverse();

            Json(MessagesResponse {
                status: "success".to_string(),
                messages,
            })
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Json(MessagesResponse {
                status: "error".to_string(),
                messages: vec![],
            })
        }
    }
}
