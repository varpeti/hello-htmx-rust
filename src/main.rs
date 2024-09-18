mod auth;
mod clients;
mod handle_websocket;
mod uuser;

use auth::hash_password;
use clients::Clients;
use handle_websocket::handle_websocket;
use std::{collections::HashMap, error::Error, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use uuid::Uuid;
use uuser::{Auth, Role, Uuser};
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
    // TODO: from env
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
    // TODO: from env
    let (db, connection) =
        tokio_postgres::connect("postgres://tstuser:tstpw@db:5432/tstdb", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Sync Tables

    Uuser::sync_table(&db).await.expect("db Uuser");
    Auth::sync_table(&db).await.expect("db Auth");

    // Default Values

    // TODO: from env
    let admin = Auth {
        id: Uuid::from_str("4640bfc0-9042-4953-8c03-e2ed9fa892e2").expect("admin auth uuid"),
        uuser: Uuser {
            id: Uuid::from_str("0420dd37-4252-48c1-b341-af1dd8e126de").expect("admin uuser uuid"),
            email: "admin@admin.admin".to_string(),
            role: Role::Company,
        },
        phc_string: hash_password("This is totaly not a secure PW 🔴 TODO FIX ME!")
            .expect("admin phc_string"),
    };

    admin.upsert(&db).await.expect("db upsert admin");

    Ok(Arc::new(db))
}
