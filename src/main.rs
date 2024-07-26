use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::{filters::ws::Message, ws::WebSocket, Filter};

#[tokio::main]
async fn main() {
    // Define a simple GET route
    let index = warp::path::end().and(warp::fs::file("./index.html"));

    // Define a WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_websocket));

    // Combine the routes
    let routes = index.or(ws_route);

    // Start the server
    println!("Server starting on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_websocket(ws: WebSocket) {
    println!("WebSocket connection established");

    // Split the socket into a sender and receive of messages.
    let (mut tx, mut rx) = ws.split();

    // Use a while let loop to receive messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                // Process the message based on its type
                if let Ok(text) = msg.to_str() {
                    println!("Received message: {}", text);

                    match serde_json::from_str::<UserMessage>(text) {
                        Ok(user_message) => {
                            // Echo the message back to the client
                            tx.send(Message::text(format!(
                                r##"<div hx-swap-oob="beforeend:#idMessage"><p>{}</p></div>"##,
                                user_message.message
                            )))
                            .await
                            .expect("Error sendig Message!")
                        }
                        Err(err) => eprintln!("Parsing error: {:?}", err),
                    }
                } else if msg.is_binary() {
                    println!("Received binary message of {} bytes", msg.as_bytes().len());
                } else if msg.is_close() {
                    println!("Received close message");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    println!("WebSocket connection closed");
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMessage {
    message: String,
    #[serde(rename = "HEADERS")]
    headers: Value,
}
