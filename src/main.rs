mod clients;
mod handle_websocket;

use clients::Clients;
use handle_websocket::handle_websocket;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Define a simple GET route to serve index.html
    let index = warp::path::end().and(warp::fs::file("./index.html"));

    let connections = Arc::new(Mutex::new(HashMap::new()));

    // Define a WebSocket route to handle everything else
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(with_clients(connections.clone()))
        .map(
            |ws: warp::ws::Ws, addr: Option<SocketAddr>, clients: Clients| {
                ws.on_upgrade(move |ws| handle_websocket(ws, addr, clients))
            },
        );

    let routes = index.or(ws_route);
    println!("Server starting on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
