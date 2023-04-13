//! Small client demo.
#![cfg(debug_assertions)]

use core::time;
use std::{
    io::BufReader,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use fourmilaby_core::{
    error::ServerError,
    message::{
        transmit::{read_message, read_message_raw, write_message, write_message_raw},
        types::{JoinMessageBody, Message, MoveMessageBody},
    },
};

pub fn client_benchmark() -> Result<(), ServerError> {
    thread::sleep(time::Duration::from_secs(4));

    let mut stream = TcpStream::connect(SocketAddr::from_str("127.0.0.1:8080").unwrap()).unwrap();

    write_message(
        &mut stream,
        &Message::Join(JoinMessageBody {
            difficulty: 1,
            player_id: None,
        }),
    )?;

    let message = read_message(&mut stream)?;

    match message {
        Message::OkMaze(ok_maze) => {
            println!(
                "as {}, in {}x{} maze",
                ok_maze.player_id, ok_maze.maze.nb_column, ok_maze.maze.nb_line
            );
        }
        _ => return Err(ServerError::Transmission("Invalid message received".into())),
    }

    let move_msg = serde_json::to_string(&Message::Move(MoveMessageBody { direction: 2 })).unwrap();

    let start = Instant::now();

    let mut count = 0u64;

    let mut recv_stream = BufReader::new(stream.try_clone().unwrap());

    // Stress-test server
    while start.elapsed().as_secs() < 10 {
        write_message_raw(&mut stream, move_msg.as_bytes())?;
        read_message_raw(&mut recv_stream)?;

        count += 1;
    }

    println!("{} msg/sec", count as f64 / 10.0);

    Ok(())
}

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
