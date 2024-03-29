//! The client session management as seen from the lobby.
use std::sync::mpsc::{self, Receiver, Sender};

use uuid::Uuid;

use crate::{
    error::ServerError,
    game::{GameSessionMessage, GameSessionMessageKind},
    lobby::message::{LobbyMessage, MatchmakingInfo},
    message::types::{JoinMessageBody, Message, OkMazeMessageBody},
    protocols::PlayerChannel,
};

/// Instanciate a client negociation with with the lobby.
pub fn client_session_init<C: PlayerChannel>(
    mut client: C,
    channel: Sender<LobbyMessage>,
) -> Result<(), ServerError> {
    let res = match client.read_message() {
        // Received join
        Ok(Message::Join(body)) => client_session_negociate(client.clone_instance(), channel, body),

        // Received something else
        // Send Unexpected message error to client.
        Ok(unexpected) => {
            client.write_message(&Message::Unexpected {
                expected: vec!["join".into()],
                received: unexpected.into(),
            })?;

            Err(ServerError::Transmission(
                "Unexpected message received".into(),
            ))
        }

        // Something went wrong during read_message()
        Err(err) => Err(err),
    };

    if let Err(err) = &res {
        client.write_message(&Message::Error(err.clone())).ok();
    }

    let shutdown_res = client.stop();

    if let Err(err) = res.and(shutdown_res) {
        eprintln!("Client session terminated : {}", &err);

        Err(err)
    } else {
        Ok(())
    }
}

/// Negociate a game session with the lobby.
fn client_session_negociate<C: PlayerChannel>(
    mut client: C,
    sender: Sender<LobbyMessage>,
    body: JoinMessageBody,
) -> Result<(), ServerError> {
    println!("Started client session (UUID = {:?})", body.player_id);

    let (tx, rx) = mpsc::channel();

    sender.send(LobbyMessage::Matchmaking(body, tx.into()))?;

    // receive matchmaking information from lobby
    // that way, we get the ok maze that also contains the player UUID used internally
    match rx.recv() {
        // Ok with OkMaze
        Ok(MatchmakingInfo::JoinedGame(uuid, game_session)) => {
            client.write_message(&Message::OkMaze(OkMazeMessageBody {
                maze: game_session.maze.clone(),
                player_id: uuid,
            }))?;

            // Split the client session in two parts:
            //  - receiving messages from game session and forwarding them to socket (sender)
            //  - receiving messages from socket and forwarding them to game session (receiver)
            //
            // Put the sender in another thread, and reuse the current thread for the receiver.

            // Create a channel between the game session and the sending thread.
            let (sender_tx, sender_rx) = mpsc::channel::<Message>();

            // Duplicate the socket into a write end.
            let mut sender_client = client.clone_instance();

            // Fetch the game session channel then notify the game session of this new player.
            let game_session_channel = game_session.channel.lock()?.clone();

            game_session_channel.send(GameSessionMessage(
                uuid,
                GameSessionMessageKind::InitializePlayer(sender_tx),
            ))?;

            std::thread::Builder::new()
                .name(format!(
                    "client send {}",
                    client.get_name().unwrap_or_default()
                ))
                .spawn(move || client_session_send_loop(&mut sender_client, sender_rx))?;

            // Receiver loop
            client_session_recv_loop(&mut client, game_session_channel, uuid)?;
        }

        // UUID is not recognized by lobby.
        Ok(MatchmakingInfo::ExpiredUuid) => client.write_message(&Message::Error(
            ServerError::Other("Invalid UUID or game doesn't exist anymore.".into()),
        ))?,

        // Internal failures.
        Ok(MatchmakingInfo::InternalFailure(e)) => client.write_message(&Message::Error(e))?,
        Err(err) => client.write_message(&Message::Error(ServerError::other(err)))?,
    };

    Ok(())
}

/// Client [`Message`] (from [`GameSessionMessage`]) receiving loop.
fn client_session_recv_loop<C: PlayerChannel>(
    client: &mut C,
    channel: Sender<GameSessionMessage>,
    uuid: Uuid,
) -> Result<(), ServerError> {
    loop {
        let msg = client.read_message()?;

        channel.send(GameSessionMessage(
            uuid,
            GameSessionMessageKind::ClientMessage(msg),
        ))?;
    }
}

/// Client [`Message`] sending loop.
fn client_session_send_loop<C: PlayerChannel>(
    client: &mut C,
    receiver: Receiver<Message>,
) -> Result<(), ServerError> {
    loop {
        let msg = receiver.recv()?;

        client.write_message(&msg)?;
    }
}
