use crate::{
    maze::{Maze, Tile},
    message::types::MoveMessageBody,
};

use super::{state::GameState, PlayerInfo};

enum Movement {
    Up,
    Down,
    Right,
    Left,
    Unknown,
}

impl From<u8> for Movement {
    fn from(m: u8) -> Self {
        match m {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Right,
            3 => Self::Left,
            _ => Self::Unknown,
        }
    }
}

/// TODO: Improve it ?
pub fn update_player_position(
    maze: &Maze,
    player: &mut PlayerInfo,
    msg: &MoveMessageBody,
) -> Option<Tile> {
    if let Some(tile) = maze.get_tile(player.position.0, player.position.1) {
        // Position considering movement (try to not underflow, may lead out of bounds; checked later)
        let (new_px, new_py) = match Movement::from(msg.direction) {
            Movement::Up => (player.position.0, player.position.1.saturating_add(1)),
            Movement::Down => (player.position.0, player.position.1.saturating_sub(1)),
            Movement::Right => (player.position.0.saturating_add(1), player.position.1),
            Movement::Left => (player.position.0.saturating_sub(1), player.position.1),
            Movement::Unknown => (player.position.0, player.position.1),
        };

        // Check if we are passing through a wall.
        let through_wall = match Movement::from(msg.direction) {
            Movement::Up => tile.wall_north(),
            Movement::Down => tile.wall_south(),
            Movement::Right => tile.wall_east(),
            Movement::Left => tile.wall_west(),
            Movement::Unknown => false,
        };

        // TODO: Check destination wall but in the other side ?

        if !through_wall {
            if let Some(tile) = maze.get_tile(new_px, new_py) {
                player.position = (new_px, new_py);
                return Some(tile);
            }
        }
    } else {
        println!(
            "buggy player position ({}, {})",
            player.position.0, player.position.1
        );
    }

    None
}

impl GameState {
    pub fn process_movement(
        &mut self,
        mut player: PlayerInfo,
        msg: &MoveMessageBody,
    ) -> PlayerInfo {
        let new_tile = update_player_position(&self.maze, &mut player, msg);

        if let Some(tile) = new_tile {
            if player.has_food && tile.is_nest() {
                player.has_food = !player.has_food;

                println!("TODO: Food put into nest");
            }

            if tile.is_food() {
                player.has_food = true;
            }
        }

        player
    }
}
