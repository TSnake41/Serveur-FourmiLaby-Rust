use std::collections::VecDeque;

use crate::{maze::Maze, message::types::MoveDirection};

/// Grid of precalculated paths
pub struct DfsPath {
    moves: Box<[MoveDirection]>,
}

/// Coordinate to explore
struct Candidate {
    distance: usize,
    coords: (u32, u32),
}

pub struct NoShortestPath;

pub fn invert_move_direction(direction: MoveDirection) -> MoveDirection {
    match direction {
        MoveDirection::North => MoveDirection::South,
        MoveDirection::South => MoveDirection::North,
        MoveDirection::East => MoveDirection::West,
        MoveDirection::West => MoveDirection::East,
    }
}

impl DfsPath {
    pub fn find_shortest(maze: &Maze) -> Result<Self, NoShortestPath> {
        let coords_to_linear = |(x, y): (u32, u32)| (x + y * maze.nb_column) as usize;

        // None ~> infinity
        let mut grid: Box<[Option<usize>]> =
            vec![None; (maze.nb_column * maze.nb_line) as usize].into_boxed_slice();

        let mut candidates = VecDeque::with_capacity(1024);
        candidates.push_back(Candidate {
            coords: (maze.nest_column, maze.nest_line),
            distance: 0,
        });

        let mut food_found = false;

        while let Some(candidate) = candidates.pop_front() {
            grid[coords_to_linear(candidate.coords)] = Some(candidate.distance);

            if let Some(tile) = maze.get_tile(candidate.coords.0, candidate.coords.1) {
                if tile.is_food() {
                    food_found = true;
                    break;
                }
            }
        }

        food_found
            .then(|| Self {
                moves: vec![].into_boxed_slice(),
            })
            .ok_or(NoShortestPath)
    }
}
