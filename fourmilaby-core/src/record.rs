//! Record replay system.
use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::{
    error::ServerError,
    game::{
        record::GameRecord, state::GameState, GameSession, GameSessionMessage,
        GameSessionMessageKind,
    },
    message::types::Message,
    protocols::ClientChannel,
};

/// Create and replay a game using [`GameRecord`], send the infos in [`std::net::TcpStream`].
pub fn replay_game<C: ClientChannel>(
    channel: C,
    game_record: GameRecord,
) -> Result<(), ServerError> {
    // Create a new game, and take its
    let info = GameSession::start_new(GameState::new(game_record.maze), false)?;
    let game_channel = info.channel.lock()?.clone();

    let (send_channel, recv_channel) = mpsc::channel::<Message>();

    // Initialize all the players of the record.
    game_record.players.iter().for_each(|uuid| {
        game_channel
            .send(GameSessionMessage(
                *uuid,
                GameSessionMessageKind::InitializePlayer(send_channel.clone()),
            ))
            .unwrap()
    });

    // Make a thread that will receive messages from game session, and forward the to the stream.
    let forward_thread =
        thread::spawn(move || replay_game_forward_thread(channel, recv_channel).unwrap());

    // Reuse this thread to send events to server considering delays.
    for record in game_record.messages.iter() {
        thread::sleep(record.delay);

        game_channel
            .send(GameSessionMessage(
                record.player,
                GameSessionMessageKind::ClientMessage(record.message.clone()),
            ))
            .unwrap()
    }

    forward_thread.join().unwrap();

    Ok(())
}

/// Pipe the messages from the [`Receiver<Message>`] into the [`std::net::TcpStream`].
pub fn replay_game_forward_thread<C: ClientChannel>(
    mut channel: C,
    receiver: Receiver<Message>,
) -> Result<(), ServerError> {
    receiver
        .into_iter()
        .for_each(|message| channel.write_message(&message).unwrap());

    Ok(())
}
