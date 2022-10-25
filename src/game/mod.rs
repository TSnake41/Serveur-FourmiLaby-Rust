use std::sync::{mpsc::Sender, Mutex};

use uuid::Uuid;

use crate::{maze::Maze, message::types::Message};

pub enum GameSessionMessageKind {
    ClientMessage(Message),
    InitializePlayer(),
    UpdateAllPlayers,
}

pub struct GameSessionMessage(pub Uuid, pub GameSessionMessageKind);

pub struct GameSessionInfo {
    pub channel: Mutex<Sender<GameSessionMessage>>,
    pub maze: Maze,
}
