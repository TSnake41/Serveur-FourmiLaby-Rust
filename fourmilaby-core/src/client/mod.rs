//! WIP: Client helpers.

use std::{
    error::Error,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    error::ServerError,
    maze::Maze,
    message::types::{JoinMessageBody, Message},
    protocols::PlayerChannel,
};

/// State of a game as seen by a client.
#[derive(Clone, Default)]
pub struct ClientGameView {
    pub maze: Maze,
    pub player_position: (u32, u32),
    pub pheromon: Arc<Box<[f32]>>,
    pub player_has_food: bool,
}

/// State of the client.
#[derive(Default, Debug)]
pub enum ClientState {
    #[default]
    Uninitialized,
    JoinPending,
    Joined,
    Dead,
}

/// A client connection to a server.
pub struct ClientInstance<C>
where
    C: PlayerChannel,
{
    pub view: ClientGameView,
    pub state: ClientState,
    pub player_uuid: Option<uuid::Uuid>,
    pub channel: C,
}

pub struct BackgroundClientInstance {
    pub sender: mpsc::Sender<Message>,
    pub view: Arc<Mutex<ClientGameView>>,

    pub receiver_thread: JoinHandle<Result<(), ServerError>>,
    pub sender_thread: JoinHandle<Result<(), ServerError>>,
}

impl<C: PlayerChannel> Drop for ClientInstance<C> {
    fn drop(&mut self) {
        if let Err(err) = self.channel.stop() {
            eprintln!("Couldn't stop channel gracefully ({err})");
        }
    }
}

impl<C: PlayerChannel> ClientInstance<C> {
    pub fn new(channel: C) -> Self {
        Self {
            view: ClientGameView::default(),
            state: ClientState::default(),
            player_uuid: None,
            channel,
        }
    }

    pub fn join(&mut self, body: JoinMessageBody) -> Result<(), ServerError> {
        if !matches!(self.state, ClientState::Uninitialized) {
            return ServerError::transmission_error(format!(
                "Invalid state {:?} for join operation.",
                self.state
            ));
        }

        self.channel
            .write_message(&Message::Join(body))
            .and_then(|_| {
                self.state = ClientState::JoinPending;
                Ok(())
            })
            .map_err(|error| {
                self.state = ClientState::Dead;
                error
            })
    }

    pub fn read_message(&mut self) -> Result<(), ServerError> {
        // Some early checks on state.
        match self.state {
            ClientState::Dead => return ServerError::transmission_error(
                "Attempting to read messages from a dead instance.",
            ),
            ClientState::Uninitialized => return ServerError::transmission_error(
                "Channel may be connected but is not initialized, you need to do a join operation first.",
            ),
            _ => ()
        }

        let message = self.channel.read_message().map_err(|error| {
            self.state = ClientState::Dead;
            error
        })?;

        match self.state {
            ClientState::JoinPending => {
                if let Message::OkMaze(ok_maze) = message {
                    self.view.maze = ok_maze.maze;
                    self.player_uuid = Some(ok_maze.player_id);

                    self.state = ClientState::Joined;

                    Ok(())
                } else {
                    self.state = ClientState::Dead;

                    ServerError::transmission_error("Unexpected message received when joining.")
                }
            }
            ClientState::Joined => {
                // Check next message
                if let Message::Info(info) = message {
                    // Update view
                    self.view.player_position = (info.player_column, info.player_line);
                    self.view.pheromon = info.pheromon;
                    self.view.player_has_food = info.player_has_food;

                    Ok(())
                } else {
                    self.state = ClientState::Dead;

                    ServerError::transmission_error(
                        "Unexpected message received during info fetching.",
                    )
                }
            }
            ClientState::Uninitialized | ClientState::Dead => unreachable!(),
        }
    }

    /// Make the client instance reading and sending messages in background.
    pub fn backgroundify(mut self) -> Result<BackgroundClientInstance, Box<dyn Error>> {
        if !matches!(self.state, ClientState::Joined) {
            return ServerError::transmission_error("Can't backgroundify a non joined instance.")?;
        }

        let move_channels = mpsc::channel();

        let mut receiver_channel = self.channel.clone_instance();
        let receiver_thread = thread::Builder::new()
            .name("Client sender".to_string())
            .spawn(move || -> Result<(), ServerError> {
                while let Ok(msg) = move_channels.1.recv() {
                    receiver_channel.write_message(&msg)?;
                }

                Ok(())
            })?;

        let view: Arc<Mutex<ClientGameView>> = Arc::new(Mutex::new(self.view.clone()));
        let thread_view = view.clone();

        let sender_thread = thread::Builder::new()
            .name("Client receiver".to_string())
            .spawn(move || -> Result<(), ServerError> {
                loop {
                    self.read_message()?;

                    let mut new_view = thread_view.lock()?;

                    new_view.pheromon = self.view.pheromon.clone();
                    new_view.player_position = self.view.player_position;
                    new_view.player_has_food = self.view.player_has_food;
                }
            })?;

        Ok(BackgroundClientInstance {
            sender: move_channels.0,
            view,
            receiver_thread,
            sender_thread,
        })
    }
}
