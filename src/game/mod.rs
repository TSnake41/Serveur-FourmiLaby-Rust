use crate::{
    error::ServerError,
    maze::Maze,
    message::types::{InfoMessageBody, Message},
};

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, sync_channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use uuid::Uuid;

/// The kind of message that can be sent to a game session channel.
pub enum GameSessionMessageKind {
    InitializePlayer(Sender<Message>),
    ClientMessage(Message),
    UpdateAllPlayers,
}

/// A game session message sent to a game session channel.
pub struct GameSessionMessage(pub Uuid, pub GameSessionMessageKind);

/// The information associated to a game session shared between lobby and the game session.
/// May be sent to a client session.
pub struct GameSessionInfo {
    pub channel: Mutex<Sender<GameSessionMessage>>,
    pub maze: Maze,
}

/// The player information
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

/// A game session.
/// This instance should be only used by a single thread.
pub struct GameSession {
    maze: Maze,
    players: HashMap<Uuid, PlayerInfo>,
    channel: Receiver<GameSessionMessage>,

    /// Must be kept held to keep alive the weak lobby's [`std::sync::Weak`] refeference.
    _info: Arc<GameSessionInfo>,

    /// pheromon may be sent though channels, use Arc::make_mut to make this object mutable
    /// as needed while not needing to duplicate the whole vector each time we need a copy
    /// of it by following a clone-on-write behaviour.
    pheromon: Arc<Box<[f32]>>,

    /// Internal instance UUID, used for debugging.
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
        if sender.send(message).is_err() {
            // We can't send message to channel, invalidate the channel.
            eprintln!("{} disconnected the session {}", uuid, session_uuid);

            channel.take();
        }
    }
}

impl GameSession {
    /// Creates a new [`GameSession`].
    pub fn new(maze: Maze) -> (Self, Arc<GameSessionInfo>) {
        let (sender, receiver) = mpsc::channel::<GameSessionMessage>();
        let info = Arc::new(GameSessionInfo {
            channel: sender.into(),
            maze: maze.clone(),
        });

        let pheromon: Vec<f32> = vec![0f32; maze.nb_column as usize * maze.nb_line as usize];

        (
            Self {
                maze,
                players: HashMap::with_capacity(8),
                uuid: Uuid::new_v4(),
                channel: receiver,
                _info: info.clone(),
                pheromon: pheromon.into_boxed_slice().into(),
            },
            info,
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
            Message::Move(move_body) => {
                assert!(player.channel.is_some(), "Channel must exist to be able to receive the feedback. Has recv client channel panicked ?");

                // TODO: game logic

                // Send player info
                try_sending_to_channel(
                    &mut player.channel,
                    Message::Info(InfoMessageBody {
                        player_column: player.position.0,
                        player_line: player.position.1,
                        player_has_food: player.has_food,
                        pheromon: self.pheromon.clone(), // TODO
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
        match self.players.get_mut(uuid) {
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
                    .insert(*uuid, PlayerInfo::new(&self.maze, sender));

                Ok(())
            }
        }
    }

    /// Run the game session loop.
    pub fn run(&mut self) -> Result<(), ServerError> {
        // Create update all thread
        let sender = self._info.channel.lock().unwrap().clone();
        thread::Builder::new()
            .name(format!("GameSession updater {}", self.uuid))
            .spawn(move || loop {
                thread::sleep(Duration::from_secs(1));
                sender
                    .send(GameSessionMessage(
                        uuid::Uuid::default(),
                        GameSessionMessageKind::UpdateAllPlayers,
                    ))
                    .unwrap();
            })
            .unwrap();

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
                                pheromon: self.pheromon.clone(), // TODO
                            }),
                            uuid,
                            &self.uuid,
                        );
                    })
                }
            }
        }
    }

    /// Start in a new thread the game session loop.
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
