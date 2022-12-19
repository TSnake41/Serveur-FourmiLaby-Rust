use core::time;
use std::{
    net::{SocketAddr, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

use crate::{
    error::ServerError,
    message::{
        transmit::{read_message, write_message},
        types::{JoinMessageBody, Message, MoveMessageBody},
    },
};

pub fn client_test() -> Result<(), ServerError> {
    thread::sleep(time::Duration::from_secs(4));
    for _ in 0..5 {
        thread::spawn(|| {
            let mut stream =
                TcpStream::connect(SocketAddr::from_str("127.0.0.1:8080").unwrap()).unwrap();

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

    Ok(())
}
