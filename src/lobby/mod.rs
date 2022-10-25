mod lobby;
mod message;
mod session;

use std::{
    collections::HashMap,
    sync::{self, Arc},
};

use uuid::Uuid;

use crate::{
    error::ServerError, game::GameSessionInfo, maze::Maze, message::types::JoinMessageBody,
};

use self::message::MatchmakingInfo;

pub struct Lobby {
    // Weak pointers allows us to know if a game session is still alive.
    // However, we will have to housekeep some of these collections to prevent memory leaking.
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
        //
        // TODO: Make a real game
        //
        let game = Arc::into_raw(Arc::new(GameSessionInfo {
            channel: sync::mpsc::channel().0.into(),
            maze: Maze::default(),
        }));

        unsafe { Arc::increment_strong_count(game) }

        let game = unsafe { Arc::from_raw(game) };

        self.games.push(Arc::downgrade(&game));

        Ok(game)
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
        // collection.drain_filter is unstable as of Rust 1.62

        // Remove all player UUID that references games that doesn't exist anymore.
        let to_remove: Box<[Uuid]> = self
            .players
            .iter()
            .filter_map(|(k, session)| {
                if session.upgrade().is_none() {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect();

        to_remove.iter().for_each(|uuid| {
            self.players.remove(uuid);
        });

        // Remove all session references for games that doesn't exist anymore.
        // This is really tricky.
        let to_remove_vec: Box<[usize]> = self
            .games
            .iter()
            .enumerate()
            .rev() // reverse to preserve indices which removing
            .filter_map(|(i, session)| {
                if session.upgrade().is_none() {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        to_remove_vec.iter().for_each(|i| {
            self.games.remove(*i);
        });
    }
}
