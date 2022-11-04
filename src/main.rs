/// Manages the ServerError type that handle all kind of errors that may happen.
mod error;

/// Set of APIs to interact with some specific external libraries.
mod ffi;

/// Maze representation and utilities.
mod maze;

/// Message and protocol implementation between client and server.
mod message;

/// The game session.
mod game;

// Lobby creation and loops.
mod lobby;

/// The client session management.
mod client;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

use error::ServerError;
use message::{
    transmit::{read_message, write_message},
    types::{JoinMessageBody, Message},
};

use crate::message::types::MoveMessageBody;

fn main() -> Result<(), ServerError> {
    std::thread::spawn(|| -> Result<_, ServerError> {
        let lobby = lobby::Lobby::new();

        lobby.run(TcpListener::bind(
            SocketAddr::from_str("0.0.0.0:8080").unwrap(),
        )?)?;

        Ok(())
    });

    std::thread::sleep(std::time::Duration::from_secs(4));

    for i in 0..1500 {
        println!("{}", i);
        thread::spawn(|| -> ! {
            let mut stream =
                TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080)).unwrap();

            println!(
                "{:?}",
                write_message(
                    &mut stream,
                    &Message::Join(JoinMessageBody {
                        difficulty: 0,
                        player_id: None,
                    }),
                )
            );

            loop {
                read_message(&mut stream).unwrap();

                for i in 1..24 {
                    write_message(
                        &mut stream,
                        &Message::Move(MoveMessageBody { direction: 0 }),
                    )
                    .unwrap();

                    thread::sleep(Duration::from_millis(30));

                    read_message(&mut stream).unwrap();
                }
            }
        });

        thread::sleep(Duration::from_millis(15));
    }

    thread::sleep(Duration::MAX);

    Ok(())
}
