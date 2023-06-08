//! Maze generation algorithms.
//! Depth-first algorithm is inspired on Vincent Brunet's C++ implementation (Polytech Tours school of engineering).
use std::collections::VecDeque;

use fastrand::Rng;

use super::Maze;
use crate::{
    config::{GeneratorConfig, NestPositioning},
    error::ServerError,
    message::types::{JoinMessageBody, MoveDirection},
};

impl Maze {
    /// Put walls on the maze hulls.
    fn generate_border(&mut self) {
        for i in 0..self.nb_column {
            // Upper wall
            self.get_tile_mut(i, 0).unwrap().set_wall_north(true);

            // Bottom wall
            self.get_tile_mut(i, self.nb_line - 1)
                .unwrap()
                .set_wall_south(true);
        }

        for i in 0..self.nb_line {
            // Left wall
            self.get_tile_mut(0, i).unwrap().set_wall_west(true);

            // Right wall
            self.get_tile_mut(self.nb_column - 1, i)
                .unwrap()
                .set_wall_east(true);
        }
    }

    /// Fill the maze with walls.
    fn fill_maze(&mut self) {
        for x in 0..self.nb_column {
            for y in 0..self.nb_line {
                if let Some(tile) = self.get_tile_mut(x, y) {
                    tile.set_wall_north(true);
                    tile.set_wall_south(true);
                    tile.set_wall_west(true);
                    tile.set_wall_east(true);
                }
            }
        }
    }

    /// Add foods pseudo-randomly to the maze.
    fn place_food(&mut self, food_count: u32, rng: &Rng) -> Result<Box<[(u32, u32)]>, ServerError> {
        // Be a little bit more conservative to prevent an eventual infinite loop.
        if food_count + 1 >= (self.nb_column.saturating_sub(1)) * (self.nb_line.saturating_sub(1)) {
            return Err(ServerError::invalid_maze("Can't place food"));
        }

        // Compute a minimum distance (squared) to nest.
        let minimum_distance2 = u32::max(self.nb_line / 3, self.nest_column / 3).pow(2);

        let mut foods = vec![];
        foods.reserve(food_count as usize);

        for _ in 0..food_count {
            let proposal = (rng.u32(0..self.nb_column), rng.u32(0..self.nb_line));

            /* Check minimum distance and already taken tile. */
            if (proposal.0 * proposal.0 + proposal.1 * proposal.1) < minimum_distance2
                || foods.contains(&proposal)
            {
                continue;
            }

            foods.push(proposal);
        }

        for food in foods.iter() {
            if let Some(tile) = self.get_tile_mut(food.0, food.1) {
                tile.set_food(true)
            }
        }

        Ok(foods.into_boxed_slice())
    }

    fn get_random_neighbor(
        &self,
        (column, line): (u32, u32),
        marked: &[bool],
        rng: &Rng,
    ) -> Option<(MoveDirection, (u32, u32))> {
        let mut directions: [MoveDirection; 4] = [
            MoveDirection::North,
            MoveDirection::South,
            MoveDirection::East,
            MoveDirection::West,
        ];
        rng.shuffle(&mut directions);

        for dir in directions {
            // Get neighbor position (if any).
            if let Some(pos) = MoveDirection::apply_movement(dir, (column, line)) {
                // Get marker
                if pos.0 < self.nb_column && pos.1 < self.nb_line // check in bounds
                    && !marked[(pos.0 + pos.1 * self.nb_column) as usize]
                {
                    return Some((dir, pos));
                }
            }
        }

        None
    }

    /// Apply a backtracking algorithm to carve the walls of the maze.
    fn backtracing_carving(
        &mut self,
        (start_column, start_line): (u32, u32),
        rng: &Rng,
    ) -> Result<(), ServerError> {
        if start_column >= self.nb_column || start_line >= self.nb_line {
            return Err(ServerError::invalid_maze(format!(
                "Unexpected start position: ({start_column}, {start_line}) (maximum: ({}, {}))",
                self.nb_column, self.nb_line
            )));
        }

        let mut marked_tiles =
            vec![false; (self.nb_line * self.nb_column) as usize].into_boxed_slice();

        let mut backtracking_queue: VecDeque<(u32, u32)> = VecDeque::new();

        backtracking_queue.push_back((start_column, start_line));
        marked_tiles[(start_column + start_line * self.nb_column) as usize] = true;

        while let Some(tile) = backtracking_queue.front() {
            if let Some((dir, neighbor)) = self.get_random_neighbor(*tile, &marked_tiles, rng) {
                // Break walls
                self.break_wall(dir, &tile, neighbor);

                marked_tiles[(neighbor.0 + neighbor.1 * self.nb_column) as usize] = true;
                backtracking_queue.push_front(neighbor);
            } else {
                // Backtrack, no unmarqued neighbors
                backtracking_queue.pop_front();
            }
        }

        // Make sure the hull still exists.
        self.generate_border();
        Ok(())
    }

    /// Break a neighbor wall.
    fn break_wall(&mut self, dir: MoveDirection, tile: &(u32, u32), neighbor: (u32, u32)) {
        match dir {
            MoveDirection::North => {
                // N S
                if let Some(tile) = self.get_tile_mut(tile.0, tile.1) {
                    tile.set_wall_north(false);
                }

                if let Some(tile) = self.get_tile_mut(neighbor.0, neighbor.1) {
                    tile.set_wall_south(false);
                }
            }
            MoveDirection::South => {
                // S N
                if let Some(tile) = self.get_tile_mut(tile.0, tile.1) {
                    tile.set_wall_south(false);
                }

                if let Some(tile) = self.get_tile_mut(neighbor.0, neighbor.1) {
                    tile.set_wall_north(false);
                }
            }
            MoveDirection::East => {
                // E W
                if let Some(tile) = self.get_tile_mut(tile.0, tile.1) {
                    tile.set_wall_east(false);
                }

                if let Some(tile) = self.get_tile_mut(neighbor.0, neighbor.1) {
                    tile.set_wall_west(false);
                }
            }
            MoveDirection::West => {
                // W E
                if let Some(tile) = self.get_tile_mut(tile.0, tile.1) {
                    tile.set_wall_west(false);
                }

                if let Some(tile) = self.get_tile_mut(neighbor.0, neighbor.1) {
                    tile.set_wall_east(false);
                }
            }
        }
    }
}

/// Generate a empty maze.
pub fn generate_empty_maze(nb_column: u32, nb_line: u32, rng: &Rng) -> Result<Maze, ServerError> {
    if nb_column == 0 || nb_line == 0 {
        return Err(ServerError::invalid_maze("Can't generate an empty maze !"));
    }

    let mut maze = Maze {
        nb_column: nb_column.into(),
        nb_line: nb_line.into(),
        nest_column: fastrand::u32(0..nb_column),
        nest_line: fastrand::u32(0..nb_line),
        tiles: vec![0u8; (nb_column * nb_line) as usize].into_boxed_slice(),
    };

    // Place the nest
    if let Some(tile) = maze.get_tile_mut(maze.nest_column, maze.nest_line) {
        tile.set_nest(true);
    }

    maze.generate_border();
    maze.place_food(1, rng)?;

    Ok(maze)
}

pub fn generate_maze_backtracking(
    (nb_column, nb_line): (u32, u32),
    (nest_column, nest_line): (u32, u32),
    nb_food: u32,
    carving_amount: u32,
    rng: &Rng,
) -> Result<Maze, ServerError> {
    if nb_column == 0 || nb_line == 0 {
        return Err(ServerError::invalid_maze("Can't generate an empty maze !"));
    }

    let mut maze = Maze {
        nb_column,
        nb_line,
        nest_column,
        nest_line,
        tiles: vec![0u8; (nb_column * nb_line) as usize].into_boxed_slice(),
    };

    // Place the nest
    if let Some(tile) = maze.get_tile_mut(maze.nest_column, maze.nest_line) {
        tile.set_nest(true);
    }

    maze.fill_maze();
    maze.place_food(nb_food, rng)?;

    for _ in 0..carving_amount {
        maze.backtracing_carving((nest_column, nest_line), rng)?;
    }

    Ok(maze)
}

pub fn generate_maze(
    config: &GeneratorConfig,
    critera: &JoinMessageBody,
    rng: &Rng,
) -> Result<Maze, ServerError> {
    let size = (
        config.column_min + (config.column_coeff * critera.difficulty as f32) as u32,
        config.line_min + (config.line_coeff * critera.difficulty as f32) as u32,
    );

    let nest_pos = match config.nest_pos {
        NestPositioning::Randomized => (rng.u32(0..size.0), rng.u32(0..size.1)),
        NestPositioning::Fixed(x, y) => (x, y),
    };

    let nb_food = config.nb_food_min + (config.nb_food_coeff * critera.difficulty as f32) as u32;

    generate_maze_backtracking(size, nest_pos, nb_food, config.carving_amount, rng)
}
