use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user_email: String,
    pub username: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "chat")]
    Chat { user_email: String, content: String },
    #[serde(rename = "join")]
    Join { user_email: String },
    #[serde(rename = "leave")]
    Leave { user_email: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub status: String,
    pub message: Option<ChatMessage>,
    pub info: Option<String>,
}

pub type Tx = broadcast::Sender<ChatMessage>;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<
        Arc<(
            Pool<Postgres>,
            tokio::sync::broadcast::Sender<crate::websocket_handler::ChatMessage>,
        )>,
    >,
) -> Response {
    ws.on_upgrade(move |socket| websocket_connection(socket, state.0.clone(), state.1.clone()))
}

async fn websocket_connection(stream: WebSocket, pool: Pool<Postgres>, tx: Tx) {
    let (mut sender, mut receiver) = stream.split();
    let mut rx = tx.subscribe();

    // Task to receive messages from the broadcast channel and send to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let response = WsResponse {
                status: "message".to_string(),
                message: Some(msg),
                info: None,
            };

            if let Ok(json) = serde_json::to_string(&response) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task to receive messages from the client and broadcast to others
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Parse the incoming message
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(ws_msg) => match ws_msg {
                    WsMessage::Chat {
                        user_email,
                        content,
                    } => {
                        // Get username from database
                        let username = get_username(&pool, &user_email).await;

                        // Store message in database
                        if let Some(user_id) = get_user_id(&pool, &user_email).await {
                            let _ = store_message(&pool, user_id, &content).await;
                        }

                        // Broadcast message to all connected clients
                        let chat_message = ChatMessage {
                            user_email: user_email.clone(),
                            username: username.unwrap_or_else(|| "Unknown".to_string()),
                            content,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };

                        let _ = tx_clone.send(chat_message);
                    }
                    WsMessage::Join { user_email } => {
                        println!("User {} joined the chat", user_email);
                    }
                    WsMessage::Leave { user_email } => {
                        println!("User {} left the chat", user_email);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

/// Gets username from DB
async fn get_username(pool: &Pool<Postgres>, email: &str) -> Option<String> {
    sqlx::query_scalar("SELECT name FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Gets user name from DB
async fn get_user_id(pool: &Pool<Postgres>, email: &str) -> Option<i32> {
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Stores message in DB
async fn store_message(
    pool: &Pool<Postgres>,
    user_id: i32,
    content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO messages (user_id, content) VALUES ($1, $2)")
        .bind(user_id)
        .bind(content)
        .execute(pool)
        .await?;
    Ok(())
}
