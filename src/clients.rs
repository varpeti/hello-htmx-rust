use futures_util::{stream::SplitSink, SinkExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::filters::ws::{Message, WebSocket};

pub type ClientId = SocketAddr;
pub type Sender = SplitSink<WebSocket, Message>;
pub type Clients = Arc<Mutex<HashMap<ClientId, Sender>>>;

pub async fn add_connection(clients: &Clients, client_id: ClientId, sender: Sender) {
    clients.lock().await.insert(client_id, sender);
}

pub async fn remove_connection(clients: &Clients, client_id: &ClientId) {
    clients.lock().await.remove(client_id);
}

pub async fn broadcast(clients: &mut Clients, filter: fn(&ClientId) -> bool, message: &str) {
    let mut clients = clients.lock().await;
    for (client_id, sender) in clients.iter_mut() {
        if filter(client_id) {
            match sender.send(Message::text(message)).await {
                Ok(_) => (),
                Err(err) => eprintln!("Unable to send message: {}", err),
            }
        }
    }
}
