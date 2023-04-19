//! The original implementation of the TCP/IP protocol for this project.
use std::{
    borrow::Cow,
    net::{Shutdown, TcpListener, TcpStream},
};

use crate::{
    error::ServerError,
    message::{transmit, types::Message},
};

use super::{LobbyListener, PlayerChannel};

impl PlayerChannel for TcpStream {
    fn read_message(&mut self) -> Result<Message, ServerError> {
        transmit::read_message(self)
    }

    fn write_message(&mut self, message: &Message) -> Result<(), ServerError> {
        transmit::write_message(self, message)
    }

    fn stop(&mut self) -> Result<(), ServerError> {
        self.shutdown(Shutdown::Both)
            .map_err(|err| ServerError::Other(err.to_string().into()))
    }

    fn clone_instance(&self) -> Self {
        self.try_clone().unwrap()
    }

    fn get_name(&self) -> Option<Cow<str>> {
        self.peer_addr()
            .and_then(|addr| Ok(addr.to_string().into()))
            .ok()
    }
}

impl LobbyListener<TcpStream> for TcpListener {
    fn accept_client(&mut self) -> Result<(TcpStream, Cow<str>), ServerError> {
        self.accept()
            .map(|(stream, addr)| (stream, addr.to_string().into()))
            .or_else(|err| ServerError::other_error(err))
    }

    fn get_binding_name(&self) -> Option<Cow<str>> {
        self.local_addr().map(|addr| addr.to_string().into()).ok()
    }
}
