use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use warp::{filters::ws::Message, ws::WebSocket};

use crate::clients::{add_connection, broadcast, remove_connection, Clients};

pub async fn handle_websocket(ws: WebSocket, addr: Option<SocketAddr>, clients: Clients) {
    println!("WebSocket connection established: {:?}", addr);

    let addr = match addr {
        Some(addr) => addr,
        None => {
            eprintln!("WebSocket without address? Closing...");
            return;
        }
    };

    // Split the socket into a sender and receive of messages.
    let (tx, mut rx) = ws.split();

    add_connection(&clients, addr, tx).await;

    // Use a while let loop to receive messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if !handle_message(msg, clients.clone()).await {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    remove_connection(&clients, &addr).await;

    println!("WebSocket connection closed");
}

async fn handle_message(msg: Message, clients: Clients) -> bool {
    if let Ok(text) = msg.to_str() {
        handle_text_message(text, clients).await;
        true
    } else if msg.is_close() {
        println!("Received close message");
        false
    } else {
        eprintln!(
            "Unexpected Type! It is binary: {}, ping: {}, pong: {}",
            msg.is_binary(),
            msg.is_ping(),
            msg.is_pong()
        );
        false
    }
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
