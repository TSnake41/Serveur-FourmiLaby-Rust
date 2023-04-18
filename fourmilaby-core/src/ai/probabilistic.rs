use crate::{
    maze::Maze,
    message::types::{InfoMessageBody, MoveDirection, MoveMessageBody},
};

use super::AntAI;

/// Probability-based AI.
/// Walk randomly.
#[derive(Default)]
pub struct ProbabilisticAnt;

impl AntAI for ProbabilisticAnt {
    fn step(&mut self, maze: &Maze, message: &InfoMessageBody) -> Option<MoveMessageBody> {
        let pos = (message.player_column, message.player_line);

        let mut dirs = [
            MoveDirection::North,
            MoveDirection::South,
            MoveDirection::West,
            MoveDirection::East,
        ];

        fastrand::shuffle(&mut dirs);

        if let Some(tile) = maze.get_tile(pos.0, pos.1) {
            for direction in dirs {
                if match direction {
                    MoveDirection::North => !tile.wall_north(),
                    MoveDirection::South => !tile.wall_south(),
                    MoveDirection::East => !tile.wall_east(),
                    MoveDirection::West => !tile.wall_west(),
                } {
                    return Some(MoveMessageBody { direction });
                }
            }
        }

        // stuck :(
        None
    }
}
