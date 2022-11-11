use crate::{
    error::ServerError,
    game::{GameSessionMessage, GameSessionMessageKind},
    lobby::message::{LobbyMessage, MatchmakingInfo},
    message::{
        transmit::{read_message, write_message},
        types::{JoinMessageBody, Message, OkMazeMessageBody},
    },
};

use std::{
    net::{Shutdown, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
};

use uuid::Uuid;

/// Instanciate a client negociation with with the lobby.
pub fn client_session_init(
    client: &mut TcpStream,
    channel: Sender<LobbyMessage>,
) -> Result<(), ServerError> {
    let res = match read_message(client) {
        // Received join
        Ok(Message::Join(body)) => client_session_negociate(client, channel, body),

        // Received something else
        // Send Unexpected message error to client.
        Ok(unexpected) => {
            let err = Err(ServerError::Transmission(
                "Unexpected message received".into(),
            ));

            write_message(
                client,
                &Message::Unexpected {
                    expected: vec!["join".into()],
                    received: unexpected.into(),
                },
            )?;

            err
        }

        // Something went wrong during read_message()
        Err(err) => {
            write_message(client, &Message::Error(err.clone()))?;
            Err(err)
        }
    };

    if let Err(err) = &res {
        write_message(client, &Message::Error(err.clone())).ok();
    }

    let shutdown_res = client
        .shutdown(Shutdown::Both)
        .map_err(|io_err| ServerError::Other(io_err.to_string().into()));

    if let Err(err) = res.and(shutdown_res) {
        eprintln!("Client session terminated : {}", &err);

        Err(err)
    } else {
        Ok(())
    }
}

/// Negociate a game session with the lobby.
fn client_session_negociate(
    client: &mut TcpStream,
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
            write_message(
                client,
                &Message::OkMaze(OkMazeMessageBody {
                    maze: game_session.maze.clone(),
                    player_id: uuid,
                }),
            )?;

            // Split the client session in two parts:
            //  - receiving messages from game session and forwarding them to socket (sender)
            //  - receiving messages from socket and forwarding them to game session (receiver)
            //
            // Put the sender in another thread, and reuse the current thread for the receiver.

            // Create a channel between the game session and the sending thread.
            let (sender_tx, sender_rx) = mpsc::channel::<Message>();

            // Duplicate the socket into a write end.
            let mut sender_client = client.try_clone().unwrap();

            // Fetch the game session channel then notify the game session of this new player.
            let game_session_channel = game_session.channel.lock()?.clone();

            game_session_channel.send(GameSessionMessage(
                uuid,
                GameSessionMessageKind::InitializePlayer(sender_tx),
            ))?;

            std::thread::Builder::new()
                .name(format!("client send {}", client.peer_addr()?))
                .spawn(move || client_session_send_loop(&mut sender_client, sender_rx))?;

            // Receiver loop
            client_session_recv_loop(client, game_session_channel, uuid)?;
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

fn client_session_recv_loop(
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

fn client_session_send_loop(
    client: &mut TcpStream,
    receiver: Receiver<Message>,
) -> Result<(), ServerError> {
    loop {
        let msg = receiver.recv()?;

        write_message(client, &msg)?;
    }
}
