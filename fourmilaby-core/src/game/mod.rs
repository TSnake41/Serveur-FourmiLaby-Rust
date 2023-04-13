//! The game session.
mod logic;
pub mod record;
pub mod state;

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use uuid::Uuid;

use crate::{
    error::ServerError,
    game::{
        record::GameRecord,
        state::{GameState, PlayerInfo},
    },
    maze::Maze,
    message::types::{InfoMessageBody, Message},
};

use self::record::GameRecordState;

/// The delay between each update all ticks.
const UPDATE_PLAYERS_DELAY: Duration = Duration::from_secs(1);

/// The delay between each pheromon update.
const PHEROMON_UPDATE_DELAY: Duration = Duration::from_secs(5);

/// The kind of message that can be sent to a game session channel.
pub enum GameSessionMessageKind {
    InitializePlayer(Sender<Message>),
    ClientMessage(Message),
    UpdateAllPlayers,
    UpdatePheromon,
}

/// A game session message sent to a game session channel.
pub struct GameSessionMessage(pub Uuid, pub GameSessionMessageKind);

/// The information associated to a game session shared between lobby and the game session.
/// May be sent to a client session.
#[derive(Debug)]
pub struct GameSessionInfo {
    pub channel: Mutex<Sender<GameSessionMessage>>,
    pub maze: Maze,
}

struct PlayerChannel(Option<Sender<Message>>);

/// A game session.
/// This instance should be only used by a single thread.
pub struct GameSession {
    players: HashMap<Uuid, PlayerChannel>,
    channel: Receiver<GameSessionMessage>,

    /// Must be kept held to keep alive the weak lobby's [`std::sync::Weak`] reference.
    _info: Arc<GameSessionInfo>,

    state: GameState,
    record_state: Option<GameRecordState>,

    /// Internal instance UUID, used for debugging.
    uuid: Uuid,
}

/// Try sending a [`Message`] to the [`PlayerChannel`] (if a channel is bound to it).
/// Otherwise, invalidates the channel.
fn try_sending_to_channel(
    channel: &mut PlayerChannel,
    message: Message,
    uuid: &Uuid,
    session_uuid: &Uuid,
) {
    // If the player has an active channel.
    if let Some(sender) = &channel.0 {
        // Try sending a info message.
        if sender.send(message).is_err() {
            // We can't send message to channel, invalidate the channel.
            eprintln!("{}: player {uuid} disconnected", session_uuid.as_braced());

            channel.0.take();
        }
    }
}

impl GameSession {
    /// Creates a new [`GameSession`].
    pub fn new(state: GameState, recorded: bool) -> (Self, Arc<GameSessionInfo>) {
        let (sender, receiver) = mpsc::channel::<GameSessionMessage>();
        let info = Arc::new(GameSessionInfo {
            channel: sender.into(),
            maze: state.maze.clone(),
        });

        // Build player channels info using state.players (assume not connected).
        let players: HashMap<Uuid, PlayerChannel> = state
            .players
            .iter()
            .map(|(uuid, _)| (*uuid, PlayerChannel(None)))
            .collect();

        // Create the record state the game if needed.
        let record_state = recorded.then(|| GameRecordState::new(state.maze.clone()));

        (
            Self {
                players,
                state,
                uuid: Uuid::new_v4(),
                channel: receiver,
                _info: info.clone(),
                record_state,
            },
            info,
        )
    }

    /// Process a client message.
    fn process_player_message(&mut self, uuid: &Uuid, message: &Message) {
        let (players, state) = (&mut self.players, &mut self.state);

        let channel = players
            .get_mut(uuid)
            .expect("Player must exist to be able to send message");

        // Track the message if we are recording.
        if let Some(state) = &mut self.record_state {
            state.track(uuid, message);
        }

        match state.process_message(uuid, message) {
            Ok(info) => {
                try_sending_to_channel(
                    channel,
                    Message::Info(InfoMessageBody {
                        player_column: info.position.0,
                        player_line: info.position.1,
                        player_has_food: info.has_food,
                        pheromon: self.state.pheromon.clone(),
                    }),
                    uuid,
                    &self.uuid,
                );
            }

            // Inform the player of an unexpected message.
            Err(ServerError::UnexpectedParameter) => {
                try_sending_to_channel(
                    channel,
                    Message::Unexpected {
                        expected: vec!["move".into()],
                        received: message.clone().into(),
                    },
                    uuid,
                    &self.uuid,
                );
            }
            Err(e) => {
                eprintln!("Internal server error: {e:?}");
            }
        }
    }

    /**
    Initialize the player [`Uuid`] using the provided [`Sender<Message>`].

    If the player already exists in the session (e.g was previously connected), reset its channel using `sender`.

    Otherwise, set player at initial nest coordinates.
    */
    fn init_player(&mut self, uuid: &Uuid, sender: Sender<Message>) -> Result<(), ServerError> {
        // Check if the player exists in the session.
        match self.players.get_mut(uuid) {
            Some(channel) => {
                // A player with this UUID exists.
                match &mut channel.0 {
                    Some(_) => {
                        // A channel is already set, a client with this UUID is already connected.
                        Err(ServerError::AlreadyConnected)
                    }
                    None => {
                        println!("{}: {} reconnected", self.uuid.as_braced(), uuid);

                        // Rebind the player channel using sender.
                        let _ = channel.0.replace(sender);

                        Ok(())
                    }
                }
            }
            None => {
                // Initialize the player info using the session maze, then add this player to the session.
                println!("{}: {} connected", self.uuid.as_braced(), uuid);

                let _ = self.players.insert(*uuid, PlayerChannel(Some(sender)));

                let _ = self
                    .state
                    .players
                    .insert(*uuid, PlayerInfo::new(&self.state.maze));

                Ok(())
            }
        }
    }

    /// Run the game session loop.
    pub fn run(&mut self) -> Result<(), ServerError> {
        let sender = self._info.channel.lock().unwrap().clone();

        // Create the update all notifier thread
        let player_updater_channel = sender.clone();
        thread::Builder::new()
            .name(format!("GameSession updater {}", self.uuid))
            .spawn(move || -> Result<(), ServerError> {
                loop {
                    thread::sleep(UPDATE_PLAYERS_DELAY);
                    player_updater_channel.send(GameSessionMessage(
                        uuid::Uuid::default(),
                        GameSessionMessageKind::UpdateAllPlayers,
                    ))?;
                }
            })
            .unwrap();

        // Create the pheromon update notifier thread
        let pheromon_updater_channel = sender;
        thread::Builder::new()
            .name(format!("Pheromon updater {}", self.uuid))
            .spawn(move || -> Result<(), ServerError> {
                loop {
                    thread::sleep(PHEROMON_UPDATE_DELAY);
                    pheromon_updater_channel.send(GameSessionMessage(
                        uuid::Uuid::default(),
                        GameSessionMessageKind::UpdatePheromon,
                    ))?;
                }
            })
            .unwrap();

        self.run_loop()
    }

    fn run_loop(&mut self) -> Result<(), ServerError> {
        loop {
            let session_msg = self.channel.recv()?;
            let (uuid, kind) = (session_msg.0, session_msg.1);

            match kind {
                GameSessionMessageKind::ClientMessage(message) => {
                    self.process_player_message(&uuid, &message)
                }
                GameSessionMessageKind::InitializePlayer(sender) => {
                    if let Err(e) = self.init_player(&uuid, sender.clone()) {
                        // Notify the player of a failure.
                        sender.send(Message::Error(e)).ok();
                    }
                }
                GameSessionMessageKind::UpdateAllPlayers => {
                    // NOTE: We may need to invalidate the player channel if a send fails.

                    //TODO: Consider another way to end the game.
                    if self.players.iter().all(|(_, channel)| channel.0.is_none()) {
                        println!("{}: No active player, stopping", self.uuid.as_braced());

                        if let Some(state) = &self.record_state {
                            println!("Record :\n{:#?}", GameRecord::from(state.clone()));
                        }

                        return Ok(());
                    }

                    self.players.iter_mut().for_each(|(uuid, channel)| {
                        if let Some(info) = self.state.players.get(uuid) {
                            try_sending_to_channel(
                                channel,
                                Message::Info(InfoMessageBody {
                                    player_column: info.position.0,
                                    player_line: info.position.1,
                                    player_has_food: info.has_food,
                                    pheromon: self.state.pheromon.clone(),
                                }),
                                uuid,
                                &self.uuid,
                            );
                        }
                    })
                }
                GameSessionMessageKind::UpdatePheromon => self.state.update_pheromon(),
            }
        }
    }

    /// Start in a new thread the game session loop.
    pub fn start_new(
        state: GameState,
        recorded: bool,
    ) -> Result<Arc<GameSessionInfo>, ServerError> {
        // TODO: Maybe make it asynchronous using lobby's channel ?

        // Send GameSessionInfo through a channel.
        let (sender, reader) = mpsc::sync_channel::<Arc<GameSessionInfo>>(0);

        let session_uuid = Uuid::new_v4();

        let _ = thread::Builder::new()
            .name(format!("Game Instance {}", session_uuid.as_braced()))
            .spawn(move || {
                let (mut session, info) = Self::new(state, recorded);
                session.uuid = session_uuid;

                sender.send(info).unwrap();

                if let Err(e) = session.run() {
                    eprintln!("{}: error {e}", session_uuid.as_braced());
                }

                println!("{}: terminated", session_uuid.as_braced());
            })?;

        let info = reader.recv()?;

        Ok(info)
    }
}
