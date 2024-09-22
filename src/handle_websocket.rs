use std::sync::Arc;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use warp::{filters::ws::Message, ws::WebSocket};

use crate::{
    auth::{login_with_password, LoginForm},
    clients::{remove_connection, ClientId, Clients},
    config::EmailConfig,
    DB,
};

pub type TX = Arc<Mutex<SplitSink<WebSocket, Message>>>;
pub type CID = Arc<Mutex<Option<ClientId>>>;

pub async fn handle_websocket(ws: WebSocket, db: DB, clients: Clients, email_config: EmailConfig) {
    println!("WebSocket connection established: {:?}", &ws);

    // Split the socket into a sender and receive of messages.
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));

    let client_id: CID = Arc::new(Mutex::new(None));

    // Use a while let loop to receive messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(json_text) = msg.to_str() {
                    handle_json_message(
                        tx.clone(),
                        db.clone(),
                        clients.clone(),
                        client_id.clone(),
                        json_text,
                    )
                    .await;
                } else if msg.is_close() {
                    println!("Received close message");
                    break;
                } else {
                    eprintln!(
                        "Unexpected Type! It is binary: {}, ping: {}, pong: {}",
                        msg.is_binary(),
                        msg.is_ping(),
                        msg.is_pong()
                    );
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    println!("Good bye {:?}!", client_id.lock().await);

    if let Some(client_id) = *client_id.lock().await {
        remove_connection(&clients, &client_id).await;
    }

    println!("WebSocket connection closed");
}

async fn handle_json_message(tx: TX, db: DB, clients: Clients, client_id: CID, json_text: &str) {
    println!("Received message: {}", json_text);
    match serde_json::from_str::<MessageTypes>(json_text) {
        Ok(msg) => match msg {
            MessageTypes::LoginWithPassword(login_form) => {
                if let Err(err) = login_with_password(login_form, tx, db, clients, client_id).await
                {
                    eprintln!("Error: Login with password failed: {}", err);
                }
            }
            MessageTypes::LoginWithEmail(login_form) => todo!(),
            MessageTypes::ChatMessage(msg) => {
                let mut tx = tx.lock().await;
                if let Err(err) = tx
                    .send(Message::text(format!(
                        r##"<div hx-swap-oob="beforeend:#idMessage"><p>{}</p><br/></div>"##,
                        msg.chat_message
                    )))
                    .await
                {
                    eprintln!("Sending error: {:?}", err);
                }
            }
        },
        Err(err) => eprintln!("Parsing error: {:?}", err),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum MessageTypes {
    LoginWithPassword(LoginForm),
    LoginWithEmail(LoginForm),
    ChatMessage(ChatMessage),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChatMessage {
    chat_message: String,
}
