use crate::{error::ServerError, game::GameSessionInfo, message::types::JoinMessageBody};

use std::sync::{mpsc::Sender, Arc, Mutex};

use uuid::Uuid;

/// Message sent by the lobby thread to a client thread to indicate that
/// the client has joined (or not) the game (specified by JoinedGame).
pub enum MatchmakingInfo {
    JoinedGame(Uuid, Arc<GameSessionInfo>),
    ExpiredUuid,
    InternalFailure(ServerError),
}

/// Message sent by the client thread or housekeeping timer thread to the lobby thread.
pub enum LobbyIPCMessage {
    Matchmaking(JoinMessageBody, Mutex<Sender<MatchmakingInfo>>),
    Housekeep,
}
