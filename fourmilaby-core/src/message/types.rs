//! Message structures.
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{error::ServerError, maze::Maze};

/// Each movement a player can do.
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MoveDirection {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl Into<(i32, i32)> for MoveDirection {
    fn into(self) -> (i32, i32) {
        match self {
            MoveDirection::North => (0, -1),
            MoveDirection::South => (0, 1),
            MoveDirection::East => (1, 0),
            MoveDirection::West => (-1, 0),
        }
    }
}

/// Message received by the server by the client in the lobby to initiate the matchmaking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMessageBody {
    /// Asked difficulty.
    pub difficulty: u32,

    /// Optional player UUID for reconnecting session.
    pub player_id: Option<uuid::Uuid>,
}

/// Message sent by the server to the client in the lobby to prepare the client to join the game session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkMazeMessageBody {
    pub maze: Maze,
    pub player_id: uuid::Uuid,
}

/// Message sent by the server to the client that contains the current game view of the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoMessageBody {
    pub player_column: u32,
    pub player_line: u32,
    pub player_has_food: bool,
    #[allow(clippy::redundant_allocation)]
    // Needs to be boxed for Arc::make_mut() (makes [f32] Clone).
    pub pheromon: Arc<Box<[f32]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveMessageBody {
    pub direction: MoveDirection,
}

/// Enumeration of all the possible messages formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "camelCase")]
pub enum Message {
    Join(JoinMessageBody),
    OkMaze(OkMazeMessageBody),
    Info(InfoMessageBody),
    Error(ServerError),
    Move(MoveMessageBody),
    Unexpected {
        expected: Vec<Box<str>>,
        received: Box<Message>,
    },
}
