mod clients;
mod handle_websocket;

use clients::Clients;
use handle_websocket::handle_websocket;
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use warp::Filter;

type DB = Arc<tokio_postgres::Client>;

#[tokio::main]
async fn main() {
    let db = connect_db().await.expect("Could not connect to Database!");

    // Define a simple GET route to serve index.html
    let index = warp::path::end().and(warp::fs::file("./index.html"));

    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Define a WebSocket route to handle everything else
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_db(db))
        .and(with_clients(clients))
        .map(|ws: warp::ws::Ws, db: DB, clients: Clients| {
            ws.on_upgrade(move |ws| handle_websocket(ws, db, clients))
        });

    let routes = index.or(ws_route);
    println!("Server starting on http://localhost:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn connect_db() -> Result<DB, Box<dyn Error>> {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("postgres://tstuser:tstpw@db:5432/tstdb", NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query("SELECT $1::TEXT", &[&"db test"]).await?;

    // And then check that we got back the same string we sent over.
    let value: &str = rows[0].get(0);
    println!("assert: 'db test' = '{}'", value);
    assert_eq!(value, "db test");

    Ok(Arc::new(client))
}
