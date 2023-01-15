//! Small client demo.
use async_std::{net::TcpStream, task};
use core::time;
use std::{
    net::SocketAddr,
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{
    error::ServerError,
    message::{
        transmit::{read_message, write_message},
        types::{JoinMessageBody, Message, MoveMessageBody},
    },
};

pub async fn client_benchmark() -> Result<(), ServerError> {
    task::sleep(time::Duration::from_secs(4)).await;

    let mut stream = TcpStream::connect(SocketAddr::from_str("127.0.0.1:8080").unwrap())
        .await
        .unwrap();

    write_message(
        &mut stream,
        &Message::Join(JoinMessageBody {
            difficulty: 1,
            player_id: None,
        }),
    )
    .await?;

    let message = read_message(&mut stream).await?;

    match message {
        Message::OkMaze(ok_maze) => {
            println!(
                "as {}, in {}x{} maze",
                ok_maze.player_id, ok_maze.maze.nb_column, ok_maze.maze.nb_line
            );
        }
        _ => return Err(ServerError::Transmission("Invalid message received".into())),
    }

    let move_msg = Message::Move(MoveMessageBody { direction: 2 });

    let start = Instant::now();

    let mut count = 0u64;

    // Stress-test server
    while start.elapsed().as_secs() < 10 {
        write_message(&mut stream, &move_msg).await?;
        read_message(&mut stream).await?;

        count += 1;
    }

    println!("{} msg/sec", count as f64 / 10.0);

    Ok(())
}

pub async fn client_test() -> Result<(), ServerError> {
    task::sleep(time::Duration::from_secs(4)).await;

    for _ in 0..5 {
        task::spawn(async {
            let mut stream = TcpStream::connect(SocketAddr::from_str("127.0.0.1:8080").unwrap())
                .await
                .unwrap();

            write_message(
                &mut stream,
                &Message::Join(JoinMessageBody {
                    difficulty: 0,
                    player_id: None,
                }),
            )
            .await
            .unwrap();

            for _ in 0..5 {
                read_message(&mut stream).await.unwrap();

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
                    .await
                    .unwrap();

                    task::sleep(Duration::from_millis(40)).await;

                    let info = read_message(&mut stream).await.unwrap();

                    if let Message::Info(body) = info {
                        println!("{body:?}");
                    }
                }

                read_message(&mut stream).await.unwrap();
            }
        });

        task::sleep(Duration::from_millis(15)).await;
    }

    Ok(())
}
