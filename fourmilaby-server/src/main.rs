mod client_test;

use std::net::{SocketAddr, TcpListener};

use fourmilaby_core::{config, error::ServerError, lobby};

fn main() -> Result<(), ServerError> {
    let config = config::load_config(None).expect("Unable to load config file.");

    let lobby = lobby::Lobby::new(config.lobby);
    lobby.run(TcpListener::bind(SocketAddr::new(config.ip, config.port))?)
}
