//! Record replay system.
use async_std::{
    channel::{self, Receiver},
    net::TcpStream,
    task,
};

use crate::{
    error::ServerError,
    game::record::GameRecord,
    game::{state::GameState, GameSession, GameSessionMessage, GameSessionMessageKind},
    message::{transmit::write_message, types::Message},
};

/// Create and replay a game using [`GameRecord`], send the infos in [`TcpStream`].
async fn replay_game(stream: TcpStream, game_record: GameRecord) -> Result<(), ServerError> {
    // Create a new game, and take its
    let info = GameSession::start_new(GameState::new(game_record.maze), false)?;
    let game_channel = info.channel.clone();

    let (send_channel, recv_channel) = channel::unbounded::<Message>();

    // Initialize all the players of the record.
    for uuid in game_record.players.iter() {
        game_channel
            .send(GameSessionMessage(
                *uuid,
                GameSessionMessageKind::InitializePlayer(send_channel.clone()),
            ))
            .await
            .unwrap()
    }

    // Make a thread that will receive messages from game session, and forward the to the stream.
    let forward_thread = task::spawn(async move {
        replay_game_forward_thread(stream, recv_channel)
            .await
            .unwrap()
    });

    // Reuse this thread to send events to server considering delays.
    for record in game_record.messages.iter() {
        task::sleep(record.delay).await;

        game_channel
            .send(GameSessionMessage(
                record.player,
                GameSessionMessageKind::ClientMessage(record.message.clone()),
            ))
            .await
            .unwrap()
    }

    forward_thread.await;

    Ok(())
}

/// Pipe the messages from the [`Receiver<Message>`] into the [`TcpStream`].
async fn replay_game_forward_thread(
    mut stream: TcpStream,
    receiver: Receiver<Message>,
) -> Result<(), ServerError> {
    while let Ok(message) = receiver.try_recv() {
        write_message(&mut stream, &message).await.unwrap()
    }

    Ok(())
}
