///! Lobby creation and loops.
pub mod message;

use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{
        self,
        mpsc::{self, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    client,
    error::ServerError,
    external::generator::{generate_maze, ParamMaze},
    game::{state::GameState, GameSession, GameSessionInfo},
    maze::generate_basic_maze,
    message::types::JoinMessageBody,
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
}

impl Lobby {
    /// Create a new empty lobby.
    pub fn new() -> Self {
        Lobby {
            games: Vec::with_capacity(4),
            players: HashMap::with_capacity(64),
        }
    }

    fn lobby(send: &Sender<LobbyMessage>, listener: TcpListener) -> ! {
        loop {
            let (mut stream, addr) = listener.accept().unwrap();
            println!("[{addr}] connected");

            let channel = send.clone();

            // Create a new client session
            thread::Builder::new()
                .name(format!("client session {}", addr))
                .spawn(move || client::client_session_init(&mut stream, channel))
                .unwrap();
        }
    }

    pub fn run(mut self, listener: TcpListener) -> Result<(), ServerError> {
        println!("Lobby loop listening on {}", listener.local_addr()?);

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
            .spawn(move || Self::lobby(&sender, listener))?;

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
        let maze = if cfg!(feature = "external_maze_gen") {
            generate_maze(
                &(ParamMaze {
                    nb_column: 5 + 3 * critera.difficulty,
                    nb_line: 4 + 3 * critera.difficulty,
                    nest_column: 1,
                    nest_line: 1,
                    nb_food: 1 + critera.difficulty / 4,
                    difficulty: critera.difficulty,
                }),
            )?
        } else {
            generate_basic_maze(5)?
        };

        let session = GameSession::start_new(GameState::new(maze), true);

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
                if let Some(session) = self.games.first() {
                    MatchmakingInfo::JoinedGame(Uuid::new_v4(), session.upgrade().unwrap())
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
