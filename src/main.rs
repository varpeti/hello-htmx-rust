mod auth;
mod clients;
mod config;
mod handle_websocket;
mod uuser;

use auth::hash_password;
use clients::Clients;
use config::{AdminAuthConfig, Config, DatabaseConfig, EmailConfig};
use handle_websocket::handle_websocket;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use uuser::{Auth, Role, Uuser};
use warp::Filter;

type DB = Arc<tokio_postgres::Client>;

#[tokio::main]
async fn main() {
    let config = Config::new();

    let db = connect_db(&config.database)
        .await
        .expect("Could not connect to Database!");

    init_db(db.clone(), &config.admin).await;

    // Define a simple GET route to serve index.html
    let index = warp::path::end().and(warp::fs::file("./index.html"));

    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Define a WebSocket route to handle everything else
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_db(db))
        .and(with_clients(clients))
        .and(with_email_config(config.email))
        .map(
            |ws: warp::ws::Ws, db: DB, clients: Clients, email_config: EmailConfig| {
                ws.on_upgrade(move |ws| handle_websocket(ws, db, clients, email_config))
            },
        );

    let routes = index.or(ws_route);

    println!("Server starting on http://{}", config.webserver.ip_port);
    warp::serve(routes)
        .run(
            config
                .webserver
                .ip_port
                .parse::<SocketAddr>()
                .expect("Invalid IP:PORT!"),
        )
        .await;
}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_email_config(
    email_config: EmailConfig,
) -> impl Filter<Extract = (EmailConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || email_config.clone())
}

async fn connect_db(config: &DatabaseConfig) -> Result<DB, Box<dyn Error>> {
    let (db, connection) = tokio_postgres::connect(
        &format!(
            "postgres://{}:{}@localhost:5432/{}",
            config.user, config.password, config.db_name
        ),
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(Arc::new(db))
}

async fn init_db(db: DB, config: &AdminAuthConfig) {
    // Sync Tables

    Uuser::sync_table(&db).await.expect("db Uuser");
    Auth::sync_table(&db).await.expect("db Auth");

    // Default Values

    let admin = Auth {
        id: config.uuid_auth,
        uuser: Uuser {
            id: config.uuid_uuser,
            email: config.email.clone(),
            role: Role::Company,
        },
        phc_string: hash_password(&config.password).expect("admin phc_string"),
    };

    admin.upsert(&db).await.expect("db upsert admin");
}
