//! Game logic.
use std::sync::Arc;

use crate::{
    maze::{Maze, Tile},
    message::types::{MoveDirection, MoveMessageBody},
};

use super::{state::GameState, PlayerInfo};

/// Update the player position.
fn update_player_position(
    maze: &Maze,
    player: &mut PlayerInfo,
    msg: &MoveMessageBody,
) -> Option<Tile> {
    if let Some(tile) = maze.get_tile(player.position.0, player.position.1) {
        // Position considering movement (try to not underflow, may lead out of bounds; checked later)
        // Check if we are passing through a wall.
        let (new_px, new_py, through_wall) = match msg.direction {
            MoveDirection::North => (
                player.position.0,
                player.position.1.saturating_sub(1),
                tile.wall_north(),
            ),
            MoveDirection::South => (
                player.position.0,
                player.position.1.saturating_add(1),
                tile.wall_south(),
            ),
            MoveDirection::East => (
                player.position.0.saturating_add(1),
                player.position.1,
                tile.wall_east(),
            ),
            MoveDirection::West => (
                player.position.0.saturating_sub(1),
                player.position.1,
                tile.wall_west(),
            ),
        };

        if !through_wall {
            let dest_tile = maze.get_tile(new_px, new_py);

            // Check the other side of the wall at destination.
            let through_wall_dest = if let Some(t) = &dest_tile {
                match msg.direction {
                    MoveDirection::North => t.wall_south(),
                    MoveDirection::South => t.wall_north(),
                    MoveDirection::East => t.wall_west(),
                    MoveDirection::West => t.wall_east(),
                }
            } else {
                // Out of bounds
                true
            };

            #[cfg(debug_assertions)]
            if through_wall_dest {
                // There should be a wall (or no tile) in the opposite direction
                println!(
                    "Missing wall at ({new_px} {new_py}), from ({} {})",
                    player.position.0, player.position.1
                );
            }

            // Update the player position if there is a tile at the
            // destination (player must land somewhere in the grid).
            if dest_tile.is_some() {
                player.position = (new_px, new_py);
                return dest_tile;
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
    /// Process the movement of a player, triggering appropriate actions.
    pub fn process_movement(
        &mut self,
        mut player: PlayerInfo,
        msg: &MoveMessageBody,
    ) -> PlayerInfo {
        let new_tile = update_player_position(&self.maze, &mut player, msg);

        if let Some(tile) = new_tile {
            // The player actually moved succesfully, if it carries food, drop pheromon at his position.
            if player.has_food {
                self.drop_pheromon(player.position)
            }

            if player.has_food && tile.is_nest() {
                player.has_food = !player.has_food;

                //println!("TODO: Food put into nest");
            }

            if tile.is_food() {
                player.has_food = true;
            }
        }

        player
    }

    /// Update the pheromon level of each tile of the maze (see [`GameState`].pheromon).
    pub fn update_pheromon(&mut self) {
        // sigma_ij <- (1 - evaporation) * sigma_ij
        const EVAPORATION_RATE: f32 = 0.1;

        Arc::make_mut(&mut self.pheromon).iter_mut().for_each(|s| {
            *s = f32::clamp(*s * (1f32 - EVAPORATION_RATE), 0f32, 1f32);
        });
    }

    /// Drop pheromon on `position`, do nothing if `position` is out of bounds.
    pub fn drop_pheromon(&mut self, position: (u32, u32)) {
        // Add pheromon on the tile.
        if let Some(level) = Arc::make_mut(&mut self.pheromon)
            .get_mut((position.0 + position.1 * self.maze.nb_column) as usize)
        {
            const PHEROMON_DROP_AMOUNT: f32 = 0.2;

            *level = (*level + PHEROMON_DROP_AMOUNT).clamp(0f32, 1f32);
        }
    }
}
