//! Artifical intelligences implementations.
pub mod probabilistic;
pub mod dfs;

use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use uuid::Uuid;

use crate::{
    error::ServerError,
    game::{GameSessionMessage, GameSessionMessageKind},
    maze::Maze,
    message::types::{InfoMessageBody, Message, MoveMessageBody},
};

/// A Ant AI.
pub trait AntAI: Default + Send + 'static {
    fn step(&mut self, maze: &Maze, message: &InfoMessageBody) -> Option<MoveMessageBody>;
}

/// A group of artifical ants connected to a server.
pub struct AntGroup<AI: AntAI> {
    ants: HashMap<Uuid, (AI, Receiver<Message>)>,
    game_channel: Sender<GameSessionMessage>,
    maze: Maze,
}

impl<AI: AntAI> AntGroup<AI> {
    pub fn new(
        count: usize,
        game_channel: Sender<GameSessionMessage>,
        maze: Maze,
    ) -> Result<Self, ServerError> {
        let mut ants = HashMap::with_capacity(count);

        // Create and connect all ants
        for _ in 0..count {
            let uuid = Uuid::new_v4();
            let (sender, receiver) = mpsc::channel();

            ants.insert(uuid, (AI::default(), receiver));

            game_channel.send(GameSessionMessage(
                uuid,
                GameSessionMessageKind::InitializePlayer(sender),
            ))?;
        }

        Ok(AntGroup {
            ants,
            game_channel,
            maze,
        })
    }

    pub fn step(&mut self) -> Result<(), ServerError> {
        for (uuid, (ai, receiver)) in self.ants.iter_mut() {
            // Get all latest messages.
            let mut latest_message = None;
            while let Ok(message) = receiver.try_recv() {
                latest_message = Some(message);
            }

            if let Some(Message::Info(info)) = latest_message {
                if let Some(movement) = ai.step(&self.maze, &info) {
                    self.game_channel.send(GameSessionMessage(
                        *uuid,
                        GameSessionMessageKind::ClientMessage(Message::Move(movement)),
                    ))?;
                }
            }
        }

        Ok(())
    }

    fn run(mut self, period: Duration) -> Result<(), ServerError> {
        loop {
            self.step()?;
            thread::sleep(period);
        }
    }

    pub fn start(
        self,
        period: Duration,
    ) -> Result<JoinHandle<Result<(), ServerError>>, std::io::Error> {
        thread::Builder::new()
            .name("AI Group".to_string())
            .spawn(move || self.run(period))
    }
}
