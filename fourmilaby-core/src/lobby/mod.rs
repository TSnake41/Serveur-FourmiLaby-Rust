//! Lobby creation and loops.
mod handler;
pub mod message;

use std::{
    collections::HashMap,
    sync::{
        self,
        mpsc::{self, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    config::LobbyConfig,
    error::ServerError,
    game::{state::GameState, GameSession, GameSessionInfo},
    maze::generator::generate_maze,
    message::types::JoinMessageBody,
    protocols::{ClientChannel, LobbyListener},
};
use message::{LobbyMessage, MatchmakingInfo};

use uuid::Uuid;

const LOBBY_HOUSEKEEP_DELAY: Duration = Duration::from_secs(5);

pub struct Lobby {
    // Weak pointers allows us to know if a game session is still alive.
    // However, we will have to housekeep those collections to prevent memory from leaking
    // by unfreed weak pointers.
    games: Vec<sync::Weak<GameSessionInfo>>,
    players: HashMap<Uuid, sync::Weak<GameSessionInfo>>,
    config: LobbyConfig,
    rng: fastrand::Rng,
}

impl Lobby {
    /// Create a new empty lobby.
    pub fn new(config: LobbyConfig) -> Self {
        Lobby {
            games: Vec::with_capacity(4),
            players: HashMap::with_capacity(64),
            config,
            rng: fastrand::Rng::new(),
        }
    }

    fn lobby<C: ClientChannel, L: LobbyListener<C>>(
        send: &Sender<LobbyMessage>,
        mut listener: L,
    ) -> ! {
        loop {
            let (stream, addr) = listener.accept_client().unwrap();
            println!("[{addr}] connected");

            let channel = send.clone();

            // Create a new client session
            thread::Builder::new()
                .name(format!("client session {}", addr))
                .spawn(move || handler::client_session_init(stream, channel))
                .unwrap();
        }
    }

    pub fn run<C: ClientChannel, L: LobbyListener<C>>(
        mut self,
        listener: L,
    ) -> Result<(), ServerError> {
        if let Some(name) = listener.get_binding_name() {
            println!("Lobby loop listening on {name}");
        } else {
            println!("Lobby loop listening");
        }

        let (sender, receiver) = mpsc::channel::<LobbyMessage>();

        let housekeep_sender = sender.clone();

        let _housekeep_thread = thread::Builder::new()
            .name(String::from("housekeep thread"))
            .spawn(move || loop {
                thread::sleep(LOBBY_HOUSEKEEP_DELAY);
                housekeep_sender.send(LobbyMessage::Housekeep).unwrap();
            })?;

        let _accept_thread = thread::Builder::new()
            .name(String::from("lobby accept"))
            .spawn(move || -> ! { Self::lobby(&sender, listener) })?;

        loop {
            let msg = receiver.recv().unwrap();

            match msg {
                LobbyMessage::Matchmaking(body, channel) => {
                    let info = self.find_suitable_game(&body);

                    channel.lock().unwrap().send(info.clone())?;

                    // Register player UUID if it gets connected.
                    if let MatchmakingInfo::JoinedGame(uuid, session) = info {
                        self.players.insert(uuid, Arc::downgrade(&session));
                    }
                }
                LobbyMessage::Housekeep => self.housekeep(),
            }
        }
    }

    /// Get the player game session.
    fn get_player_game(&self, player_uuid: &Uuid) -> Option<Arc<GameSessionInfo>> {
        self.players
            .get(player_uuid)
            .and_then(|session| session.upgrade())
    }

    // TODO: Player limit test ? Is player allowed ?

    fn create_new_game(
        &mut self,
        critera: &JoinMessageBody,
    ) -> Result<Arc<GameSessionInfo>, ServerError> {
        let maze = generate_maze(&self.config.generator, critera, &self.rng)?;
        // TODO: Make a better API, consider modifying critera.

        let session = GameSession::start_new(GameState::new(maze), self.config.record_games);

        if let Ok(info) = &session {
            // Add the game to the list.
            self.games.push(Arc::downgrade(info));
        }

        session
    }

    /// Find a suitable game for the JoinMessage, try to reconnect to session if UUID is specified in message.
    fn find_suitable_game(&mut self, join_message: &JoinMessageBody) -> MatchmakingInfo {
        match join_message.player_id {
            // Try to reconnect player to session.
            Some(uuid) => match self.get_player_game(&uuid) {
                Some(session) => MatchmakingInfo::JoinedGame(uuid, session),
                None => MatchmakingInfo::ExpiredUuid,
            },

            // Create a new session.
            None => {
                // TODO: Find matching session, use proper matchmaking
                if let Some(session) = self.games.first().and_then(|session| session.upgrade()) {
                    MatchmakingInfo::JoinedGame(Uuid::new_v4(), session)
                } else {
                    match self.create_new_game(join_message) {
                        Ok(game) => MatchmakingInfo::JoinedGame(Uuid::new_v4(), game),
                        Err(err) => MatchmakingInfo::InternalFailure(err),
                    }
                }
            }
        }
    }

    fn housekeep(&mut self) {
        // Remove all player UUID that references games that doesn't exist anymore.
        self.players
            .retain(|_, session| session.upgrade().is_some());

        // Remove all session references for games that doesn't exist anymore.
        self.games.retain(|session| session.upgrade().is_some());
    }
}
