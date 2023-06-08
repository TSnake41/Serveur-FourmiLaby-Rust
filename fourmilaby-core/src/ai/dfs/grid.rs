use std::collections::VecDeque;

use crate::{maze::Maze, message::types::MoveDirection};

/// Grid of precalculated paths
pub struct DfsPath {
    path: Box<[MoveDirection]>,
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

        let mut food_pos = None;

        while let Some(candidate) = candidates.pop_front() {
            grid[coords_to_linear(candidate.coords)] = Some(candidate.distance);

            if let Some(tile) = maze.get_tile(candidate.coords.0, candidate.coords.1) {
                if tile.is_food() {
                    food_pos = Some(candidate.coords);
                    break;
                }

                // Check each directions
                for direction in tile.get_walkable_directions() {
                    if let Some(new_coords) =
                        MoveDirection::apply_movement(direction, candidate.coords)
                    {
                        let dest = &mut grid[coords_to_linear(new_coords)];

                        match &dest {
                            Some(dest_distance) if *dest_distance > candidate.distance => {
                                *dest = Some(candidate.distance + 1);
                            }
                            Some(_) => (),
                            None => candidates.push_back(Candidate {
                                distance: candidate.distance + 1,
                                coords: new_coords,
                            }),
                        }
                    }
                }
            }
        }

        food_pos
            .and_then(|mut pos| {
                // Make path from food
                let mut partial_path = VecDeque::new();
                let mut nest_pos = (maze.nest_column, maze.nest_line);

                while pos != nest_pos {
                    // Take neighbor with minimal distance.
                    if let Some(directions) = maze
                        .get_tile(pos.0, pos.1)
                        .and_then(|tile| Some(tile.get_walkable_directions()))
                    {
                        let (dir, new_pos, _) = directions
                            .iter()
                            .map(|dir| {
                                let new_pos = MoveDirection::apply_movement(*dir, pos).unwrap();

                                // Annotate all directions with distance.
                                (
                                    *dir,
                                    new_pos,
                                    grid[coords_to_linear(pos)].unwrap_or(usize::MAX), // consider unreachable if no value
                                )
                            })
                            .fold((MoveDirection::North, pos, usize::MAX), |a, b| {
                                // Only consider the direction with lowest distance.
                                if a.2 < b.2 {
                                    a
                                } else {
                                    b
                                }
                            });

                        pos = new_pos;
                        partial_path.push_back(invert_move_direction(dir));
                    }
                }

                Some(Self {
                    path: partial_path.into_iter().collect(),
                })
            })
            .ok_or(NoShortestPath)
    }
}
