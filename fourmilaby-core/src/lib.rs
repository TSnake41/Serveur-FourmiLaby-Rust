//! # Fourmilaby Rust Server
//!
//! This is a heavily work in progress research project made in Rust that is meant to be ant game/simulator
//! that focuses on high performance and modularity. This project is meant to be used along <https://github.com/Akahara/AntsGame/>.
pub mod client;
pub mod config;
pub mod error;
pub mod game;
pub mod lobby;
pub mod maze;
pub mod message;
pub mod protocols;
pub mod record;
pub mod ai;