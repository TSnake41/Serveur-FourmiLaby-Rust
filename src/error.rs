use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerError {
    InvalidMaze(Box<str>),
    Transmission(Box<str>),
    SerializerError(Box<str>),
    Other(Box<str>),
}

impl ServerError {
    pub fn transmission<S>(str: S) -> Self
    where
        S: Into<Box<str>>,
    {
        ServerError::Transmission(str.into())
    }

    pub fn transmission_error<S, R>(str: S) -> Result<R, ServerError>
    where
        S: Into<Box<str>>,
    {
        Err(Self::transmission(str))
    }

    pub fn invalid_maze<S>(str: S) -> Self
    where
        S: Into<Box<str>>,
    {
        ServerError::InvalidMaze(str.into())
    }

    pub fn invalid_maze_error<S, R>(str: S) -> Result<R, ServerError>
    where
        S: Into<Box<str>>,
    {
        Err(Self::invalid_maze(str))
    }

    pub fn other<E>(err: E) -> ServerError
    where
        E: Error,
    {
        ServerError::Other(err.to_string().into_boxed_str())
    }

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

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::InvalidMaze(msg) => write!(f, "InvalidMaze: {}", msg),
            ServerError::Transmission(msg) => write!(f, "Transmission: {}", msg),
            ServerError::Other(msg) => write!(f, "Other: {}", msg),
            ServerError::SerializerError(msg) => write!(f, "Serialization: {}", msg),
        }
    }
}

impl Error for ServerError {}
