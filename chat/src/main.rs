use warp::{http::Method, ws::{Message, WebSocket}, Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, Mutex as TokioMutex};
use futures_util::{StreamExt, SinkExt};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;

type Users = Arc<Mutex<HashMap<String, String>>>;
type Messages = Arc<TokioMutex<Vec<(String, String)>>>;

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    success: bool,
    message: String,
}

#[tokio::main]
async fn main() {
    let users = Arc::new(Mutex::new(HashMap::new()));
    let messages = Arc::new(TokioMutex::new(Vec::new()));
    load_messages_from_file(messages.clone()).await;

    let tx = Arc::new(TokioMutex::new(broadcast::channel::<(String, String)>(100).0));

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(with_users(users.clone()))
        .and_then(register_handler);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(with_users(users.clone()))
        .and_then(login_handler);

    let fetch_messages = warp::path("messages")
        .and(warp::get())
        .and(with_messages(messages.clone()))
        .and_then(fetch_messages_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_broadcast(tx.clone()))
        .and(with_messages(messages.clone()))
        .map(|ws: warp::ws::Ws, tx, messages| {
            ws.on_upgrade(move |websocket| handle_connection(websocket, tx, messages))
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST])
        .allow_headers(vec!["Content-Type"]);

    let routes = register.or(login).or(ws_route).or(fetch_messages).with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn with_broadcast(
    tx: Arc<TokioMutex<broadcast::Sender<(String, String)>>>,
) -> impl Filter<Extract = (Arc<TokioMutex<broadcast::Sender<(String, String)>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

fn with_messages(messages: Messages) -> impl Filter<Extract = (Messages,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || messages.clone())
}

async fn register_handler(body: AuthRequest, users: Users) -> Result<impl Reply, Rejection> {
    let mut users = users.lock().unwrap();
    
    if users.contains_key(&body.username) {
        Ok(warp::reply::json(&AuthResponse {
            success: false,
            message: "Username already exists".into(),
        }))
    } else {
        users.insert(body.username.clone(), body.password);
        Ok(warp::reply::json(&AuthResponse {
            success: true,
            message: "User registered successfully".into(),
        }))
    }
}


async fn login_handler(body: AuthRequest, users: Users) -> Result<impl Reply, Rejection> {
    let users = users.lock().unwrap();
    if let Some(password) = users.get(&body.username) {
        if password == &body.password {
            Ok(warp::reply::json(&AuthResponse {
                success: true,
                message: "Login successful".into(),
            }))
        } else {
            Ok(warp::reply::json(&AuthResponse {
                success: false,
                message: "Invalid password".into(),
            }))
        }
    } else {
        Ok(warp::reply::json(&AuthResponse {
            success: false,
            message: "User not found".into(),
        }))
    }
}

async fn fetch_messages_handler(messages: Messages) -> Result<impl Reply, Rejection> {
    let messages = messages.lock().await;
    Ok(warp::reply::json(&*messages))
}

async fn handle_connection(
    ws: WebSocket,
    tx: Arc<TokioMutex<broadcast::Sender<(String, String)>>>,
    messages: Messages,
) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.lock().await.subscribe();

    tokio::spawn(async move {
        while let Ok((username, msg)) = rx.recv().await {
            let full_msg = format!("{}: {}", username, msg);
            if ws_sender.send(Message::text(full_msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    if let Some((username, msg)) = text.split_once(": ") {
                        tx.lock().await.send((username.to_string(), msg.to_string())).expect("Failed to broadcast message");
                        messages.lock().await.push((username.to_string(), msg.to_string()));
                        save_messages_to_file(messages.clone()).await;
                    }
                }
            }
            Err(_) => break,
        }
    }
}

async fn save_messages_to_file(messages: Messages) {
    let messages = messages.lock().await;
    let serialized = serde_json::to_string(&*messages).expect("Failed to serialize messages");
    let mut file = File::create("messages.json").await.expect("Failed to create file");
    file.write_all(serialized.as_bytes()).await.expect("Failed to write to file");
}

async fn load_messages_from_file(messages: Messages) {
    if let Ok(mut file) = File::open("messages.json").await {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).await.is_ok() {
            if let Ok(loaded) = serde_json::from_str(&contents) {
                let mut shared_messages = messages.lock().await;
                *shared_messages = loaded;
            }
        }
    }
}


