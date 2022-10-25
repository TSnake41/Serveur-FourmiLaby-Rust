use serde::{Deserialize, Serialize};

use crate::{error::ServerError, maze::Maze};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMessageBody {
    pub difficulty: i32,
    pub player_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkMazeMessageBody {
    pub maze: Maze,
    pub player_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoMessageBody {
    pub player_column: u32,
    pub player_line: u32,
    pub player_has_food: bool,
    pub pheromon: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "camelCase")]
pub enum Message {
    Join(JoinMessageBody),
    OkMaze(OkMazeMessageBody),
    Info(InfoMessageBody),
    Error(ServerError),
    Unexpected {
        expected: Vec<Box<str>>,
        received: Box<Message>,
    },
}
