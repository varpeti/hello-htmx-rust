use std::sync::Arc;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use warp::{filters::ws::Message, ws::WebSocket};

use crate::{
    clients::{remove_connection, Clients, UserId},
    DB,
};

type TX = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub async fn handle_websocket(ws: WebSocket, db: DB, clients: Clients) {
    println!("WebSocket connection established: {:?}", &ws);

    // Split the socket into a sender and receive of messages.
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));

    let user_id: Option<UserId> = None;

    // Use a while let loop to receive messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(json_text) = msg.to_str() {
                    handle_json_message(tx.clone(), db.clone(), clients.clone(), json_text).await;
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

    if let Some(user_id) = user_id {
        remove_connection(&clients, &user_id).await;
    }

    println!("WebSocket connection closed");
}

async fn handle_json_message(tx: TX, db: DB, clients: Clients, json_text: &str) {
    println!("Received message: {}", json_text);
    match serde_json::from_str::<UserMessage>(json_text) {
        Ok(user_message) => {
            // Echo
            let mut tx = tx.lock().await;

            if let Err(err) = tx
                .send(Message::text(format!(
                    r##"<div hx-swap-oob="beforeend:#idMessage"><p>{}</p><br/></div>"##,
                    user_message.message
                )))
                .await
            {
                eprintln!("Sending error: {:?}", err);
            }
        }
        Err(err) => eprintln!("Parsing error: {:?}", err),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMessage {
    message: String,
    #[serde(rename = "HEADERS")]
    headers: Value,
}
