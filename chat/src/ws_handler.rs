use warp::ws::{Message, WebSocket};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use futures_util::{StreamExt, SinkExt};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct User {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    success: bool,
    message: String,
}

pub async fn handle_register(user: User, users: Arc<RwLock<HashMap<String, String>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut users = users.write().await;
    if users.contains_key(&user.username) {
        return Ok(warp::reply::json(&ApiResponse {
            success: false,
            message: "Username already exists".to_string(),
        }));
    }
    users.insert(user.username.clone(), user.password);
    Ok(warp::reply::json(&ApiResponse {
        success: true,
        message: "Registration successful".to_string(),
    }))
}

pub async fn handle_login(user: User, users: Arc<RwLock<HashMap<String, String>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let users = users.read().await;
    if let Some(password) = users.get(&user.username) {
        if password == &user.password {
            return Ok(warp::reply::json(&ApiResponse {
                success: true,
                message: "Login successful".to_string(),
            }));
        }
    }
    Ok(warp::reply::json(&ApiResponse {
        success: false,
        message: "Invalid username or password".to_string(),
    }))
}

pub async fn handle_connection(ws: WebSocket, tx: Arc<Mutex<broadcast::Sender<String>>>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.lock().await.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    let username_message = format!("[{}] {}", "Username", text);
                    tx.lock()
                        .await
                        .send(username_message)
                        .expect("Failed to broadcast message");
                }
            }
            Err(_) => break,
        }
    }
}
