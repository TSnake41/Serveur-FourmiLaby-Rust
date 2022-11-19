use std::sync::{mpsc::Sender, Arc, Mutex};

use uuid::Uuid;

use crate::{error::ServerError, game::GameSessionInfo, message::types::JoinMessageBody};

/// Message sent by the lobby thread to a client thread to indicate that
/// the client has joined (or not) the game (specified by [`MatchmakingInfo::JoinedGame`]).
#[derive(Clone)]
pub enum MatchmakingInfo {
    JoinedGame(Uuid, Arc<GameSessionInfo>),
    ExpiredUuid,
    InternalFailure(ServerError),
}

/// Message sent by the client thread or housekeeping timer thread to the lobby thread.
pub enum LobbyMessage {
    Matchmaking(JoinMessageBody, Mutex<Sender<MatchmakingInfo>>),
    Housekeep,
}
