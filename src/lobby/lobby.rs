use std::{
    net::TcpListener,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::{
    error::ServerError,
    lobby::{message::LobbyIPCMessage, session},
};

use super::Lobby;

fn lobby_thread(send: &Sender<LobbyIPCMessage>, listener: TcpListener) -> ! {
    loop {
        let (mut stream, addr) = listener.accept().unwrap();
        println!("{} connected", addr);

        let channel = send.clone();

        // Create a new client session
        thread::Builder::new()
            .name(format!("client session {}", addr))
            .spawn(move || session::client_session_negociation(&mut stream, channel))
            .unwrap();
    }
}

impl Lobby {
    pub fn start_lobby(mut self, listener: TcpListener) -> Result<(), ServerError> {
        println!("Lobby loop listening on {}", listener.local_addr()?);

        let (send, recv) = mpsc::channel::<LobbyIPCMessage>();

        let _accept_thread = thread::Builder::new()
            .name(String::from("lobby accept"))
            .spawn(move || lobby_thread(&send, listener))?;

        loop {
            let msg = recv.recv().unwrap();

            match msg {
                LobbyIPCMessage::Matchmaking(body, channel) => {
                    channel
                        .lock()
                        .unwrap()
                        .send(self.find_suitable_game(&body))?;
                }
                LobbyIPCMessage::Housekeep => todo!(),
            }
        }
    }
}
