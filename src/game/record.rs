//! Recording system.
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{maze::Maze, message::types::Message};

use uuid::Uuid;

/// A message from a recorded game.
#[derive(Clone, Debug)]
pub struct MessageRecord {
    pub delay: Duration,
    pub player: Uuid,
    pub message: Message,
}

/// A recorded game.
#[derive(Debug)]
pub struct GameRecord {
    pub messages: Box<[MessageRecord]>,
    pub maze: Maze,
    pub players: Box<[Uuid]>,
}

/// Freeze a [`GameRecordState`] into a [`GameRecord`].
impl From<GameRecordState> for GameRecord {
    fn from(state: GameRecordState) -> Self {
        GameRecord {
            messages: state.messages.into_boxed_slice(),
            maze: state.maze,
            players: state.players.into_iter().collect(),
        }
    }
}

/// The recording state for an active game.
/// Used to iteratively record a game.
#[derive(Clone)]
pub(super) struct GameRecordState {
    pub maze: Maze,
    pub players: HashSet<Uuid>,
    pub messages: Vec<MessageRecord>,
    pub last_message_instant: Option<Instant>,
}

impl GameRecordState {
    /// Create a new empty [`GameRecordState`].
    pub fn new(maze: Maze) -> Self {
        Self {
            messages: vec![],
            players: HashSet::new(),
            last_message_instant: None,
            maze,
        }
    }

    /// Add a [`Message`] to the [`GameRecordState`].
    pub fn track(&mut self, player: &Uuid, message: &Message) {
        // Add the player to the set if it doesn't exist.
        self.players.insert(*player);

        // Compute the time between the last message and now.
        let now = Instant::now();

        let delay = match self.last_message_instant {
            Some(instant) => now.duration_since(instant),
            None => Duration::ZERO,
        };

        self.last_message_instant = Some(now);

        self.messages.push(MessageRecord {
            delay,
            player: *player,
            message: message.clone(),
        });
    }
}
