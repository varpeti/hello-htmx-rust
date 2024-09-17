use futures_util::stream::SplitSink;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

pub type UserId = Uuid;
pub type Sender = SplitSink<WebSocket, Message>;
pub type Clients = Arc<Mutex<HashMap<UserId, Sender>>>;

pub async fn add_connection(clients: &Clients, user_id: Uuid, sender: Sender) {
    clients.lock().await.insert(user_id, sender);
}

pub async fn remove_connection(clients: &Clients, client_id: &UserId) {
    clients.lock().await.remove(client_id);
}
