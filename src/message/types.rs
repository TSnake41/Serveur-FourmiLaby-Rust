use serde::{Deserialize, Serialize};

use crate::{error::ServerError, maze::Maze};

/// Message received by the server by the client in the lobby to initiate the matchmaking.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMessageBody {
    /// Asked difficulty.
    pub difficulty: i32,

    /// Optional player UUID for reconnecting session.
    pub player_id: Option<uuid::Uuid>,
}

/// Message sent by the server to the client in the lobby to prepare the client to join the game session.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkMazeMessageBody {
    pub maze: Maze,
    pub player_id: uuid::Uuid,
}

/// Message sent by the server to the client that contains the current game view of the player.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoMessageBody {
    pub player_column: u32,
    pub player_line: u32,
    pub player_has_food: bool,
    pub pheromon: Vec<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveMessageBody {
    pub direction: u32 //TODO: Use a more appropriate type for direction (enum ?).
}

/// Enumeration of all the possible messages formats.
#[derive(Clone, Serialize, Deserialize)]
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
