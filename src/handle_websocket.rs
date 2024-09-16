use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::ws::WebSocket;

use crate::{
    clients::{broadcast, remove_connection, Clients, UserId},
    DB,
};

pub async fn handle_websocket(ws: WebSocket, db: DB, clients: Clients) {
    println!("WebSocket connection established: {:?}", &ws);

    // Split the socket into a sender and receive of messages.
    let (tx, mut rx) = ws.split();

    let user_id: Option<UserId> = None;

    // Use a while let loop to receive messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    handle_text_message(text, clients.clone()).await;
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

async fn handle_text_message(text: &str, mut clients: Clients) {
    println!("Received message: {}", text);
    match serde_json::from_str::<UserMessage>(text) {
        Ok(user_message) => {
            // Echo the message back to the client
            broadcast(
                &mut clients,
                |_client_id| true,
                format!(
                    r##"<div hx-swap-oob="beforeend:#idMessage"><p>{}</p><br/></div>"##,
                    user_message.message
                )
                .as_ref(),
            )
            .await;
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
