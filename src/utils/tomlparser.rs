use actix_web::dev::Server;
use serde::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerStuff,
    pub routing: RoutingStuff,
}

#[derive(Deserialize)]
pub struct ServerStuff {
    pub address: String,
    pub port: u16,
    pub max_payload_size: usize,
}

#[derive(Deserialize)]
pub struct RoutingStuff {
    pub allow_path_params: Vec<String>,
}

pub fn server() -> Config {
    let conffile = fs::read_to_string("./config/server.toml").unwrap();
    let server: Config = toml::from_str(conffile.as_str()).unwrap();
    return server;
}
