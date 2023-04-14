//! Traits for protocol implementation.
pub mod tcp;
pub mod tungstenite;

use std::borrow::Cow;

use crate::{error::ServerError, message::types::Message};

/// Something that can accept client instances.
pub trait LobbyListener<C>: Send + 'static {
    /// Accept a player connection.
    ///
    /// #### Return value
    /// On success, returns a tuple containing a client instance and its name.
    fn accept_client(&mut self) -> Result<(C, Cow<str>), ServerError>;

    /// Get the name of the binding (e.g bound address).
    fn get_binding_name(&self) -> Option<Cow<str>>;
}

/// A channel to some client.
pub trait ClientChannel: Send + 'static {
    /// Send a message to the client instance.
    ///
    /// #### Note
    /// See [`crate::message::transmit::read_message`]
    fn read_message(&mut self) -> Result<Message, ServerError>;

    /// Send a message to the client instance.
    ///
    /// #### Note
    /// See [`crate::message::transmit::write_message`]
    fn write_message(&mut self, message: &Message) -> Result<(), ServerError>;

    /// Gracefully stop the instance.
    fn stop(&mut self) -> Result<(), ServerError>;

    /// Duplicate the client instance handle.
    fn clone_instance(&self) -> Self;

    /// Get the name of the instance (if any).
    fn get_name(&self) -> Option<Cow<str>>;
}
