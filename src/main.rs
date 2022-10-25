use std::{
    net::{SocketAddr, TcpListener},
    str::FromStr,
};

use error::ServerError;

mod error;
mod game;
mod lobby;
mod maze;
mod message;

fn main() -> Result<(), ServerError> {
    let lobby = lobby::Lobby::new();

    lobby.start_lobby(TcpListener::bind(
        SocketAddr::from_str("0.0.0.0:8080").unwrap(),
    )?)?;

    Ok(())
}
