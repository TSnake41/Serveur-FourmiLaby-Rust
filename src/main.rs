mod client;
mod client_test;
mod error;
mod external;
mod game;
mod lobby;
mod maze;
mod message;

use std::{net::SocketAddr, str::FromStr};

use async_std::{net::TcpListener, task};
use error::ServerError;

fn main() -> Result<(), ServerError> {
    // Start basic client test.
    let client_test = task::spawn(async { client_test::client_benchmark().await });

    let lobby = lobby::Lobby::new();

    async_std::task::block_on(async move {
        lobby
            .run(TcpListener::bind(SocketAddr::from_str("0.0.0.0:8080").unwrap()).await?)
            .await
    })
}
