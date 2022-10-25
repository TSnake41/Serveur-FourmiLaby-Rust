use super::message::LobbyIPCMessage;
use crate::{
    error::ServerError,
    game::{GameSessionMessage, GameSessionMessageKind},
    lobby::message::MatchmakingInfo,
    message::{
        transmit::{read_message, write_message},
        types::{JoinMessageBody, Message, OkMazeMessageBody},
    },
};

use std::{
    net::{Shutdown, TcpStream},
    sync::mpsc::{self, Sender},
};

use uuid::Uuid;

pub fn client_session_negociation(
    client: &mut TcpStream,
    channel: Sender<LobbyIPCMessage>,
) -> Result<(), ServerError> {
    if let Err(err) = match read_message(client) {
        // Received join
        Ok(Message::Join(body)) => client_session_join(client, channel, body),

        // Received something else
        // Send Unexpected message error to client.
        Ok(unexpected) => write_message(
            client,
            &Message::Unexpected {
                expected: vec!["join".into()],
                received: unexpected.into(),
            },
        ),

        // Something went wrong during read_message()
        Err(err) => write_message(client, &Message::Error(err)),
    } {
        eprintln!("Client session terminated : {}", &err);
        write_message(client, &Message::Error(err.clone())).ok();

        client.shutdown(Shutdown::Both)?;
        return Err(err);
    }

    Ok(())
}

fn client_session_join(
    client: &mut TcpStream,
    channel: Sender<LobbyIPCMessage>,
    body: JoinMessageBody,
) -> Result<(), ServerError> {
    println!("Started client session (UUID = {:?})", body.player_id);

    let (tx, rx) = mpsc::channel();

    channel
        .send(LobbyIPCMessage::Matchmaking(body, tx.into()))
        .unwrap();

    // receive matchmaking information from lobby
    // that way, we get the ok maze that also contains the player UUID used internally
    match rx.recv() {
        // Ok with OkMaze
        Ok(MatchmakingInfo::JoinedGame(uuid, game_session)) => {
            write_message(
                client,
                &Message::OkMaze(OkMazeMessageBody {
                    maze: game_session.maze.clone(),
                    player_id: uuid,
                }),
            )?;

            // TOOD: Notify game session.

            client_session_loop(client, game_session.channel.lock()?.clone(), uuid)?;
        }

        // UUID is not recognized by lobby.
        Ok(MatchmakingInfo::ExpiredUuid) => write_message(
            client,
            &Message::Error(ServerError::Other(
                "Invalid UUID or game doesn't exist anymore.".into(),
            )),
        )?,

        // Internal failures.
        Ok(MatchmakingInfo::InternalFailure(e)) => write_message(client, &Message::Error(e))?,
        Err(err) => write_message(client, &Message::Error(ServerError::other(err)))?,
    };

    Ok(())
}

fn client_session_loop(
    client: &mut TcpStream,
    channel: Sender<GameSessionMessage>,
    uuid: Uuid,
) -> Result<(), ServerError> {
    loop {
        let msg = read_message(client)?;

        channel.send(GameSessionMessage(
            uuid,
            GameSessionMessageKind::ClientMessage(msg),
        ))?;
    }
}
