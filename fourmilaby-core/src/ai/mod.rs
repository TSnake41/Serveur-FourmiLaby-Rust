//! Artifical intelligences implementations.
pub mod probabilistic;

use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
};

use uuid::Uuid;

use crate::{
    error::ServerError,
    game::{GameSessionMessage, GameSessionMessageKind},
    maze::Maze,
    message::types::{InfoMessageBody, Message, MoveMessageBody},
};

pub trait AntAI: Default {
    fn step(&mut self, maze: &Maze, message: &InfoMessageBody) -> Option<MoveMessageBody>;
}

/// A group of artifical ants connected to a server.
pub struct AntGroup<AI: AntAI> {
    ants: HashMap<Uuid, AI>,
    game_channel: Sender<GameSessionMessage>,
    receiver: Receiver<Message>,
}

impl<AI: AntAI> AntGroup<AI> {
    pub fn new(
        count: usize,
        game_channel: Sender<GameSessionMessage>,
    ) -> Result<Self, ServerError> {
        let (sender, receiver) = mpsc::channel();

        let mut ants = HashMap::with_capacity(count);

        // Create all ants
        for _ in 0..count {
            ants.insert(Uuid::new_v4(), AI::default());
        }

        // Connect them
        for (uuid, _) in ants.iter() {
            game_channel.send(GameSessionMessage(
                *uuid,
                GameSessionMessageKind::InitializePlayer(sender.clone()),
            ))?;
        }

        Ok(AntGroup {
            ants,
            game_channel,
            receiver,
        })
    }
}
