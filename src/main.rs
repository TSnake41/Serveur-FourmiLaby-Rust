mod client;
mod client_test;
mod config;
mod error;
mod external;
mod game;
mod lobby;
mod maze;
mod message;

use std::net::{SocketAddr, TcpListener};

use error::ServerError;

fn main() -> Result<(), ServerError> {
    // Start basic client test.
    #[cfg(debug_assertions)]
    std::thread::spawn(client_test::client_test);

    let config = config::load_config(None).expect("Unable to load config file.");

    let lobby = lobby::Lobby::new(config.lobby);
    lobby.run(TcpListener::bind(SocketAddr::new(config.ip, config.port))?)
}
