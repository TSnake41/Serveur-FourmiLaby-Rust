use crate::{client, error::ServerError, lobby::message::LobbyMessage};

use super::Lobby;

use std::{
    net::TcpListener,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

const LOBBY_HOUSEKEEP_DELAY: Duration = Duration::from_secs(5);

impl Lobby {
    fn lobby(send: &Sender<LobbyMessage>, listener: TcpListener) -> ! {
        loop {
            let (mut stream, addr) = listener.accept().unwrap();
            println!("{} connected", addr);

            let channel = send.clone();

            // Create a new client session
            thread::Builder::new()
                .name(format!("client session {}", addr))
                .spawn(move || client::client_session_init(&mut stream, channel))
                .unwrap();
        }
    }

    pub fn run(mut self, listener: TcpListener) -> Result<(), ServerError> {
        println!("Lobby loop listening on {}", listener.local_addr()?);

        let (sender, receiver) = mpsc::channel::<LobbyMessage>();

        let housekeep_sender = sender.clone();

        let _housekeep_thread = thread::Builder::new()
            .name(String::from("housekeep thread"))
            .spawn(move || loop {
                thread::sleep(LOBBY_HOUSEKEEP_DELAY);
                housekeep_sender.send(LobbyMessage::Housekeep).unwrap();
            })?;

        let _accept_thread = thread::Builder::new()
            .name(String::from("lobby accept"))
            .spawn(move || Self::lobby(&sender, listener))?;

        loop {
            let msg = receiver.recv().unwrap();

            match msg {
                LobbyMessage::Matchmaking(body, channel) => {
                    channel
                        .lock()
                        .unwrap()
                        .send(self.find_suitable_game(&body))?;
                }
                LobbyMessage::Housekeep => self.housekeep(),
            }
        }
    }
}
