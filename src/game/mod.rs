use crate::{
    error::ServerError,
    maze::Maze,
    message::types::{InfoMessageBody, Message},
};

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use uuid::Uuid;

pub enum GameSessionMessageKind {
    ClientMessage(Message),
    InitializePlayer(Sender<Message>),
    UpdateAllPlayers,
}

pub struct GameSessionMessage(pub Uuid, pub GameSessionMessageKind);

pub struct GameSessionInfo {
    pub channel: Mutex<Sender<GameSessionMessage>>,
    pub maze: Maze,
}

struct PlayerInfo {
    channel: Option<Sender<Message>>,
    position: (u32, u32),
    has_food: bool,
}

impl PlayerInfo {
    /// Create a player information using maze informations.
    fn new(maze: &Maze, channel: Sender<Message>) -> Self {
        Self {
            channel: Some(channel),
            position: (maze.nest_column, maze.nest_line),
            has_food: false,
        }
    }
}

pub struct GameSession {
    maze: Maze,
    players: HashMap<Uuid, PlayerInfo>,
    channel: Receiver<GameSessionMessage>,

    /// Internal instance UUID
    uuid: Uuid,
}

/// Try sending [message] to channel (if it exists), otherwise, invalidates the channel.
fn try_sending_to_channel(
    channel: &mut Option<Sender<Message>>,
    message: Message,
    uuid: &Uuid,
    session_uuid: &Uuid,
) {
    // If the player has an active channel.
    if let Some(sender) = channel {
        // Try sending a info message.
        if let Err(e) = sender.send(message) {
            // We can't send message to channel, invalidate the channel.
            eprintln!("{} disconnected of session {}", uuid, session_uuid);

            channel.take();
        }
    }
}

impl GameSession {
    /// Creates a new [`GameSession`].
    pub fn new(maze: Maze) -> (Self, Arc<GameSessionInfo>) {
        let (sender, receiver) = mpsc::channel::<GameSessionMessage>();

        (
            Self {
                maze: maze.clone(),
                players: HashMap::with_capacity(8),
                uuid: Uuid::new_v4(),
                channel: receiver,
            },
            GameSessionInfo {
                channel: sender.into(),
                maze: maze.clone(),
            }
            .into(),
        )
    }

    /// Process a client message.
    fn process_client_message(&mut self, uuid: &Uuid, message: &Message) {
        let player = self
            .players
            .get_mut(uuid)
            .expect("Player must exists to be able to send message");

        /*
        NOTE: Due to player being mutably borrowed, we can't modify `self.players` directly,
              in case we want to modify players in some way, we must use self.channel
              to alter the game state in one of the next frames.

        This is a guarantee this processing will not have side effects to the
        other players, and potentially, to the player we are actually processing.

        Note that it could be circunvented by allowing interior mutability through a [`UnsafeCell`].
        */

        match message {
            Message::Move(move_info) => {
                assert!(player.channel.is_some(), "Channel must exist to be able to receive the feedback. Has recv client channel panicked ?");

                // TODO: game logic

                // Send player info
                try_sending_to_channel(
                    &mut player.channel,
                    Message::Info(InfoMessageBody {
                        player_column: player.position.0,
                        player_line: player.position.1,
                        player_has_food: player.has_food,
                        pheromon: vec![], // TODO
                    }),
                    uuid,
                    &self.uuid,
                );
            }
            _ => {
                try_sending_to_channel(
                    &mut player.channel,
                    Message::Unexpected {
                        expected: vec!["move".into()],
                        received: message.clone().into(),
                    },
                    uuid,
                    &self.uuid,
                );
            }
        }
    }

    /**
    Initialize the player [uuid] using the provided [sender].

    If the player already exists in the session (e.g was previously connected), reset its channel using [sender].

    Otherwise, set player at initial nest coordinates.
    */
    fn init_player(&mut self, uuid: &Uuid, sender: Sender<Message>) -> Result<(), ServerError> {
        // Check if the player exists in the session.
        match self.players.get_mut(&uuid) {
            Some(player) => {
                // A player with this UUID exists.
                match &mut player.channel {
                    Some(_) => {
                        // A channel is already set, a client with this UUID is already connected.
                        Err(ServerError::AlreadyConnected)
                    }
                    None => {
                        // Rebind the player channel using sender.
                        let _ = player.channel.replace(sender);

                        Ok(())
                    }
                }
            }
            None => {
                // Initialize the player info using the session maze, then add this player to the session.
                let _ = self
                    .players
                    .insert(uuid.to_owned(), PlayerInfo::new(&self.maze, sender).into());

                Ok(())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        loop {
            let session_msg = self.channel.recv()?;
            let (uuid, kind) = (session_msg.0, session_msg.1);

            match kind {
                GameSessionMessageKind::ClientMessage(message) => {
                    self.process_client_message(&uuid, &message)
                }
                GameSessionMessageKind::InitializePlayer(sender) => {
                    if let Err(e) = self.init_player(&uuid, sender.clone()) {
                        // Notify the player of a failure.
                        sender.send(Message::Error(e)).ok();
                    }
                }
                GameSessionMessageKind::UpdateAllPlayers => {
                    // We may need to invalidate the player channel if a send fails.
                    self.players.iter_mut().for_each(|(uuid, info)| {
                        try_sending_to_channel(
                            &mut info.channel,
                            Message::Info(InfoMessageBody {
                                player_column: info.position.0,
                                player_line: info.position.1,
                                player_has_food: info.has_food,
                                pheromon: vec![], // TODO
                            }),
                            uuid,
                            &self.uuid,
                        );
                    })
                }
            }
        }
    }

    pub fn start_new(maze: Maze) -> Result<Arc<GameSessionInfo>, ServerError> {
        // TODO: Make it asynchronous using lobby's channel ?

        // Send SessionInfo through a channel.
        let (sender, reader) = mpsc::sync_channel::<Arc<GameSessionInfo>>(1);

        let session_uuid = Uuid::new_v4();

        let _ = thread::Builder::new()
            .name(format!("Game Instance {}", session_uuid.as_braced()))
            .spawn(move || {
                let (mut session, info) = Self::new(maze);
                session.uuid = session_uuid;

                sender.send(info).unwrap();

                if let Err(e) = session.run() {
                    eprintln!("GameSession error : {}", e);
                }
            })?;

        let info = reader.recv()?;

        Ok(info)
    }
}
