mod error;
mod external;
mod maze;
mod message;
mod game;
mod lobby;
mod client;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    thread,
    time::{self, Duration},
};

use error::ServerError;
use message::{
    transmit::{read_message, write_message},
    types::{JoinMessageBody, Message, MoveMessageBody},
};

fn main() -> Result<(), ServerError> {
    thread::spawn(|| -> Result<_, ServerError> {
        let lobby = lobby::Lobby::new();

        lobby.run(TcpListener::bind(
            SocketAddr::from_str("0.0.0.0:8080").unwrap(),
        )?)?;

        Ok(())
    });

    thread::sleep(time::Duration::from_secs(4));

    for i in 0..1 {
        thread::spawn(|| {
            let mut stream =
                TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080)).unwrap();

            write_message(
                &mut stream,
                &Message::Join(JoinMessageBody {
                    difficulty: 0,
                    player_id: None,
                }),
            )
            .unwrap();

            for _ in 0..5 {
                read_message(&mut stream).unwrap();

                for i in 0..=20 {
                    write_message(
                        &mut stream,
                        &Message::Move(MoveMessageBody {
                            direction: match i {
                                0..=4 => 2,   // Right
                                5..=9 => 0,   // Down
                                10..=14 => 3, // Left
                                15..=20 => 1, // Up
                                _ => 5,
                            },
                        }),
                    )
                    .unwrap();

                    thread::sleep(Duration::from_millis(40));

                    let info = read_message(&mut stream).unwrap();

                    if let Message::Info(body) = info {
                        println!("{body:?}");
                    }
                }

                read_message(&mut stream).unwrap();
            }
        });

        thread::sleep(Duration::from_millis(15));
    }

    thread::sleep(Duration::MAX);

    Ok(())
}
