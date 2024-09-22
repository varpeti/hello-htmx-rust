use futures_util::stream::SplitSink;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

pub type ClientId = Uuid;
pub type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;
pub type Clients = Arc<Mutex<HashMap<ClientId, Sender>>>;

pub async fn add_connection(
    clients: Clients,
    client_id: ClientId,
    sender: Sender,
) -> Result<(), Box<dyn Error>> {
    clients.lock().await.insert(client_id, sender.clone());
    println!(
        "Info: New uuser ({:?}) added to clients ({:?})",
        client_id,
        clients.lock().await.keys()
    );
    Ok(())
}

pub async fn remove_connection(clients: &Clients, client_id: &ClientId) {
    println!(
        "Info: Removed uuser ({:?}) from clients ({:?})",
        client_id,
        clients.lock().await.keys()
    );
    clients.lock().await.remove(client_id);
}
