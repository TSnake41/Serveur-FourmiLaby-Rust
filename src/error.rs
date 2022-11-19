//! Manages the [`ServerError`] type that handle all kind of errors that may happen.
use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

/// Represents any error that can happen on the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerError {
    InvalidMaze(Box<str>),
    Transmission(Box<str>),
    SerializerError(Box<str>),
    AlreadyConnected,
    UnexpectedParameter,
    Other(Box<str>),
}

impl ServerError {
    /// Create a [`ServerError::Transmission`]  [`ServerError`].
    pub fn transmission<S>(str: S) -> Self
    where
        S: Into<Box<str>>,
    {
        ServerError::Transmission(str.into())
    }

    /// Create a [`ServerError::Transmission`] error [`Result`].
    pub fn transmission_error<S, R>(str: S) -> Result<R, ServerError>
    where
        S: Into<Box<str>>,
    {
        Err(Self::transmission(str))
    }

    /// Create a invalid maze [`ServerError`].
    pub fn invalid_maze<S>(str: S) -> Self
    where
        S: Into<Box<str>>,
    {
        ServerError::InvalidMaze(str.into())
    }

    /// Create a [`ServerError::InvalidMaze`] error [`Result`].
    pub fn invalid_maze_error<S, R>(str: S) -> Result<R, ServerError>
    where
        S: Into<Box<str>>,
    {
        Err(Self::invalid_maze(str))
    }

    /// Map an [`Error`] into a [`ServerError`]
    pub fn other<E>(err: E) -> ServerError
    where
        E: Error,
    {
        ServerError::Other(err.to_string().into_boxed_str())
    }

    /// Map an [`Error`] into a [`ServerError`] as a [`Result`]
    pub fn other_error<E, R>(err: E) -> Result<R, ServerError>
    where
        E: Error,
    {
        Err(Self::other(err))
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::SerializerError(err.to_string().into_boxed_str())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::Transmission(err.to_string().into_boxed_str())
    }
}

impl<T> From<std::sync::PoisonError<T>> for ServerError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        ServerError::Other(format!("{:?}", err).into_boxed_str())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for ServerError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        ServerError::Other(format!("{:?}", err).into_boxed_str())
    }
}

impl From<std::sync::mpsc::RecvError> for ServerError {
    fn from(err: std::sync::mpsc::RecvError) -> Self {
        ServerError::Other(format!("{:?}", err).into_boxed_str())
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::InvalidMaze(msg) => write!(f, "InvalidMaze: {msg}"),
            ServerError::Transmission(msg) => write!(f, "Transmission: {msg}"),
            ServerError::Other(msg) => write!(f, "Other: {msg}"),
            ServerError::SerializerError(msg) => write!(f, "Serialization: {msg}"),
            ServerError::UnexpectedParameter => write!(f, "Unexpected parameter encountered"),
            ServerError::AlreadyConnected => {
                write!(f, "A client with this UUID is already connected !")
            }
        }
    }
}

impl Error for ServerError {}
