//! Maze generation algorithms.
//! Depth-first algorithm is inspired on Vincent Brunet's C++ implementation (Polytech Tours school of engineering).
use std::collections::VecDeque;

use fastrand::Rng;

use super::Maze;
use crate::{
    config::{GeneratorConfig, NestPositioning},
    error::ServerError,
    message::types::JoinMessageBody,
};

#[derive(Clone, Copy)]
enum Movement {
    Up,
    Down,
    Right,
    Left,
}

impl From<Movement> for (i32, i32) {
    fn from(value: Movement) -> Self {
        match value {
            Movement::Up => (0, 1),
            Movement::Down => (0, -1),
            Movement::Right => (1, 0),
            Movement::Left => (-1, 0),
        }
    }
}

impl Movement {
    fn apply_movement(movement: Movement, (column, line): (u32, u32)) -> Option<(u32, u32)> {
        let (dir_column, dir_line) = movement.into();

        match (
            column.checked_add_signed(dir_column),
            line.checked_add_signed(dir_line),
        ) {
            (Some(c), Some(l)) => Some((c, l)),
            _ => None, // underflowed
        }
    }
}

impl Maze {
    /// Put walls on the maze hulls.
    fn generate_hull(&mut self) {
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
            self.get_tile_mut(self.nb_column - 1, 0)
                .unwrap()
                .set_wall_east(true);
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
    ) -> Option<(Movement, (u32, u32))> {
        let mut directions: [Movement; 4] = [
            Movement::Up,
            Movement::Down,
            Movement::Right,
            Movement::Left,
        ];
        rng.shuffle(&mut directions);

        for dir in directions {
            // Get neighbor position (if any).
            if let Some(pos) = Movement::apply_movement(dir, (column, line)) {
                // Get marker
                match (
                    marked.get((pos.0 + pos.1 * self.nb_column) as usize),
                    pos.0 < self.nb_column && pos.1 < self.nb_line, // check out of bounds
                ) {
                    // Not marked and in bounds : OK
                    (Some(false), true) => return Some((dir, pos)),

                    // Out of bounds or already marked : KO
                    _ => (),
                }
            }
        }

        None
    }

    /// Apply the Depth-First Algorithm to carve the walls of the maze.
    fn df_carving(
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

        let mut tiles_queue: VecDeque<(u32, u32)> = VecDeque::new();

        tiles_queue.push_back((start_column, start_line));

        while let Some(tile) = tiles_queue.front() {
            if let Some((dir, neighbor)) = self.get_random_neighbor(*tile, &marked_tiles, rng) {
                // Break walls
                match dir {
                    Movement::Up => {
                        // N S
                        self.get_tile_mut(tile.0, tile.1)
                            .unwrap()
                            .set_wall_north(false);
                        self.get_tile_mut(neighbor.0, neighbor.1)
                            .unwrap()
                            .set_wall_south(false);
                    }
                    Movement::Down => {
                        // S N
                        self.get_tile_mut(tile.0, tile.1)
                            .unwrap()
                            .set_wall_south(false);
                        self.get_tile_mut(neighbor.0, neighbor.1)
                            .unwrap()
                            .set_wall_north(false);
                    }
                    Movement::Right => {
                        // E W
                        self.get_tile_mut(tile.0, tile.1)
                            .unwrap()
                            .set_wall_east(false);
                        self.get_tile_mut(neighbor.0, neighbor.1)
                            .unwrap()
                            .set_wall_west(false);
                    }
                    Movement::Left => {
                        // W E
                        self.get_tile_mut(tile.0, tile.1)
                            .unwrap()
                            .set_wall_west(false);
                        self.get_tile_mut(neighbor.0, neighbor.1)
                            .unwrap()
                            .set_wall_east(false);
                    }
                }

                tiles_queue.push_back(neighbor);
                marked_tiles[(neighbor.0 + neighbor.1 * self.nb_column) as usize] = true;
            } else {
                // Remove the actual tile, no unmarqued neighbors
                tiles_queue.pop_front();
            }
        }

        Ok(())
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

    maze.generate_hull();
    maze.place_food(1, rng)?;

    Ok(maze)
}

pub fn generate_df_maze(
    (nb_column, nb_line): (u32, u32),
    (nest_column, nest_line): (u32, u32),
    nb_food: u32,
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

    maze.generate_hull();
    maze.place_food(nb_food, rng)?;
    maze.df_carving((nest_column, nest_line), rng)?;

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

    generate_df_maze(size, nest_pos, nb_food, rng)
}
