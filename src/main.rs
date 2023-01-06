mod client;
mod client_test;
mod error;
mod external;
mod game;
mod lobby;
mod maze;
mod message;

use std::{
    net::{SocketAddr, TcpListener},
    str::FromStr,
    thread,
};

use error::ServerError;

fn main() -> Result<(), ServerError> {
    // Start basic client test.
    thread::spawn(client_test::client_test);

    let lobby = lobby::Lobby::new();

    lobby.run(TcpListener::bind(
        SocketAddr::from_str("0.0.0.0:8080").unwrap(),
    )?)
}
