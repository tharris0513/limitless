use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use axum_extra::{headers, TypedHeader};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct ChatState {
    pub tx: broadcast::Sender<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub username: String,
    pub text: String,
    pub timestamp: String,
}

impl ChatState {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<Arc<ChatState>>,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("WebSocket connection from: {user_agent}");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<ChatState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Username storage
    let mut username = String::from("Anonymous");

    // Spawn task to receive broadcast messages and send to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Serialize message to JSON
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    tracing::warn!("Failed to send message to client");
                    break;
                }
            }
        }
        tracing::debug!("Send task ended");
    });

    // Clone state for the receive task
    let tx = state.tx.clone();

    // Spawn task to receive messages from this client and broadcast
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        tracing::debug!("Received text message: {}", text);

                        // Check if it's a ping message
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            if value.get("type").and_then(|v| v.as_str()) == Some("ping") {
                                tracing::debug!("Received JSON ping, ignoring");
                                continue;
                            }
                        }

                        // Try to parse as ChatMessage
                        if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                            // Store username for this connection
                            if !chat_msg.username.is_empty() {
                                username = chat_msg.username.clone();
                            }

                            // Add timestamp
                            chat_msg.timestamp = chrono::Utc::now().to_rfc3339();

                            // Broadcast to all connected clients
                            if tx.send(chat_msg).is_err() {
                                tracing::error!("Failed to broadcast message");
                                break;
                            }
                        }
                    }
                    Message::Ping(_) => {
                        tracing::debug!("Received ping");
                        // Pong is automatically sent by axum
                    }
                    Message::Pong(_) => {
                        tracing::debug!("Received pong");
                    }
                    Message::Close(frame) => {
                        tracing::info!("Client {} requested close: {:?}", username, frame);
                        break;
                    }
                    Message::Binary(_) => {
                        tracing::warn!("Received unexpected binary message");
                    }
                },
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        tracing::info!("Receive task ended for user: {}", username);
        username
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            tracing::debug!("Send task completed first");
            recv_task.abort();
        },
        result = (&mut recv_task) => {
            tracing::debug!("Receive task completed first");
            if let Ok(user) = result {
                tracing::info!("User {} disconnected", user);
            }
            send_task.abort();
        },
    };

    tracing::info!("WebSocket connection fully closed");
}
