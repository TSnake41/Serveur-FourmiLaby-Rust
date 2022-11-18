//! The game state.

use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{error::ServerError, maze::Maze, message::types::Message};

/// The player information (position, status, ...).
#[derive(Clone, Copy)]
pub struct PlayerInfo {
    pub position: (u32, u32),
    pub has_food: bool,
}

impl PlayerInfo {
    /// Create a player information using maze informations.
    pub fn new(maze: &Maze) -> Self {
        Self {
            position: (maze.nest_column, maze.nest_line),
            has_food: false,
        }
    }
}

/// Game state structure.
/// This structure contains the immediate and serializable state of a game, that will change during the game life.
pub struct GameState {
    /// Consider PlayerInfo as immutable in functions.
    pub players: HashMap<Uuid, PlayerInfo>,
    pub maze: Maze,

    /// pheromon may be sent though channels, use Arc::make_mut to make this object mutable
    /// as needed while not needing to duplicate the whole vector each time we need a copy
    /// of it by following a clone-on-write behaviour.
    #[allow(clippy::redundant_allocation)]
    // Needs to be boxed for Arc::make_mut() (makes [f32] Clone).
    pub pheromon: Arc<Box<[f32]>>,
}

impl GameState {
    /// Create a new initial game state from a specific [`Maze`].
    pub fn new(maze: Maze) -> Self {
        let pheromon = vec![0f32; maze.nb_column as usize * maze.nb_line as usize];

        GameState {
            maze,
            players: HashMap::with_capacity(5),
            pheromon: pheromon.into_boxed_slice().into(),
        }
    }

    /// Process a [`Message`] from the player pointed by its [`Uuid`].
    pub fn process_message(
        &mut self,
        uuid: &Uuid,
        msg: &Message,
    ) -> Result<PlayerInfo, ServerError> {
        // Do note that this is a copy of the player info.
        let player_info = *self
            .players
            .get(uuid)
            .expect("Player is missing from the game state.");

        match msg {
            Message::Move(move_msg) => {
                let new_player_info = self.process_movement(player_info, move_msg);

                // Update player info.
                *self
                    .players
                    .get_mut(uuid)
                    .expect("Player has disappeared during process_movement() ?") = new_player_info;

                Ok(new_player_info)
            }
            _ => Err(ServerError::UnexpectedParameter),
        }
    }
}
