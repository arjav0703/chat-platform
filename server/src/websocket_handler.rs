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
use sha2::{Digest, Sha256};

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
    #[serde(rename = "auth")]
    Auth { 
        email: String, 
        password: String 
    },
    #[serde(rename = "chat")]
    Chat { 
        content: String 
    },
    #[serde(rename = "join")]
    Join,
    #[serde(rename = "leave")]
    Leave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub status: String,
    pub message: Option<ChatMessage>,
    pub info: Option<String>,
}

#[derive(Debug, Clone)]
struct AuthenticatedUser {
    email: String,
    username: String,
    user_id: i32,
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
    
    // Wait for authentication message first
    let authenticated_user = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(WsMessage::Auth { email, password }) => {
                    match authenticate_user(&pool, &email, &password).await {
                        Some(user) => {
                            // Send success response
                            let response = WsResponse {
                                status: "authenticated".to_string(),
                                message: None,
                                info: Some(format!("Welcome, {}!", user.username)),
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                            println!("User {} ({}) authenticated", user.username, user.email);
                            Some(user)
                        }
                        None => {
                            // Send error response
                            let response = WsResponse {
                                status: "error".to_string(),
                                message: None,
                                info: Some("Authentication failed: Invalid email or password".to_string()),
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                            None
                        }
                    }
                }
                _ => {
                    // Send error response
                    let response = WsResponse {
                        status: "error".to_string(),
                        message: None,
                        info: Some("First message must be authentication".to_string()),
                    };
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = sender.send(Message::Text(json)).await;
                    }
                    None
                }
            }
        }
        _ => None,
    };

    // If authentication failed, close the connection
    let user = match authenticated_user {
        Some(u) => u,
        None => {
            let _ = sender.close().await;
            return;
        }
    };

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
    let user_clone = user.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Parse the incoming message
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(ws_msg) => match ws_msg {
                    WsMessage::Chat { content } => {
                        // Store message in database
                        let _ = store_message(&pool, user_clone.user_id, &content).await;

                        // Broadcast message to all connected clients
                        let chat_message = ChatMessage {
                            user_email: user_clone.email.clone(),
                            username: user_clone.username.clone(),
                            content,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };

                        let _ = tx_clone.send(chat_message);
                    }
                    WsMessage::Join => {
                        println!("User {} joined the chat", user_clone.email);
                    }
                    WsMessage::Leave => {
                        println!("User {} left the chat", user_clone.email);
                    }
                    WsMessage::Auth { .. } => {
                        // Ignore subsequent auth messages
                        eprintln!("Received auth message after authentication");
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
    
    println!("User {} disconnected", user.email);
}

/// Authenticates a user and returns their information if successful
async fn authenticate_user(pool: &Pool<Postgres>, email: &str, password: &str) -> Option<AuthenticatedUser> {
    let password_hash = hash_password(password);
    
    let result = sqlx::query_as::<_, (i32, String, String)>(
        "SELECT id, name, email FROM users WHERE email = $1 AND password_hash = $2"
    )
    .bind(email)
    .bind(&password_hash)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some((user_id, username, email))) => Some(AuthenticatedUser {
            email,
            username,
            user_id,
        }),
        _ => None,
    }
}

/// Hash password using SHA256
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
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
