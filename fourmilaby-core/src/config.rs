//! Lobby and game configuration.
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
};

use serde::{Deserialize, Serialize};

use crate::error::ServerError;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum NestPositioning {
    Randomized,
    Fixed(u32, u32),
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub column_min: u32,
    pub column_coeff: f32,
    pub line_min: u32,
    pub line_coeff: f32,
    pub nb_food_min: u32,
    pub nb_food_coeff: f32,
    pub carving_amount: u32,

    pub basic_generator_size: u32,
    pub nest_pos: NestPositioning,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            column_min: 5,
            column_coeff: 3.0,
            line_min: 4,
            line_coeff: 3.0,
            nb_food_min: 1,
            nb_food_coeff: 0.25,
            basic_generator_size: 6,
            carving_amount: 2,

            nest_pos: NestPositioning::Fixed(1, 1),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LobbyConfig {
    pub record_games: bool,
    pub generator: GeneratorConfig,
}

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            record_games: false,
            generator: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub ip: IpAddr,
    pub port: u16,
    pub lobby: LobbyConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 8080,
            lobby: Default::default(),
        }
    }
}

const DEFAULT_CONFIG_PATH: &str = "config.json";

pub fn load_config(config_path: Option<&str>) -> Result<ServerConfig, ServerError> {
    let path_str = config_path.unwrap_or(DEFAULT_CONFIG_PATH);

    match fs::read_to_string(path_str) {
        Ok(content) => Ok(serde_json::from_str::<ServerConfig>(content.as_str())?),
        Err(err) => {
            println!("Unable to read config ({err}), regenerate config.");

            let config = ServerConfig::default();

            fs::write(path_str, serde_json::to_string_pretty(&config).unwrap())
                .expect("Unable to regenerate config file.");

            Ok(config)
        }
    }
}
