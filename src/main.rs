/// Manages the ServerError type that handle all kind of errors that may happen.
mod error;

/// Set of APIs to interact with some specific external libraries.
mod ffi;

/// Maze representation and utilities.
mod maze;

/// Message and protocol implementation between client and server.
mod message;

mod game;
mod lobby;

use std::{
    net::{SocketAddr, TcpListener},
    str::FromStr,
};

use error::ServerError;

fn main() -> Result<(), ServerError> {
    let lobby = lobby::Lobby::new();

    lobby.run(TcpListener::bind(
        SocketAddr::from_str("0.0.0.0:8080").unwrap(),
    )?)?;

    Ok(())
}
