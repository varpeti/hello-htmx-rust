use std::fs;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub webserver: WebserverConfig,
    pub database: DatabaseConfig,
    pub admin: AdminAuthConfig,
    pub email: EmailConfig,
}

impl Config {
    pub fn new() -> Config {
        let config_string =
            fs::read_to_string("./config.toml").expect("The config.toml file not found!");
        toml::from_str(&config_string).expect("Failed to parse config.toml file!")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebserverConfig {
    pub ip_port: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub db_name: String,
    pub user: String,
    pub password: String,
    pub port: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminAuthConfig {
    pub uuid_auth: Uuid,
    pub uuid_uuser: Uuid,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub sender_email: String,
    pub smtp: String,
    pub username: String,
    pub password: String,
}
