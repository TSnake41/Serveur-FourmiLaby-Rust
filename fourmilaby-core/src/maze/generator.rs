//! Maze generation algorithms.
use super::Maze;
use crate::error::ServerError;

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
    fn place_food(&mut self, food_count: u32) -> Result<(), ServerError> {
        // Be a little bit more conservative to prevent an eventual infinite loop.
        if food_count + 1 >= (self.nb_column.saturating_sub(1)) * (self.nb_line.saturating_sub(1)) {
            return Err(ServerError::invalid_maze("Can't place food"));
        }

        // Compute a minimum distance (squared) to nest.
        let minimum_distance2 = u32::max(self.nb_line / 3, self.nest_column / 3).pow(2);

        let mut foods = vec![];
        foods.reserve(food_count as usize);

        for _ in 0..food_count {
            let proposal = (
                fastrand::u32(0..self.nb_column),
                fastrand::u32(0..self.nb_line),
            );

            /* Check minimum distance and already taken tile. */
            if (proposal.0 * proposal.0 + proposal.1 * proposal.1) < minimum_distance2
                || foods.contains(&proposal)
            {
                continue;
            }

            foods.push(proposal);
        }

        for food in foods {
            if let Some(tile) = self.get_tile_mut(food.0, food.1) {
                tile.set_food(true)
            }
        }

        Ok(())
    }
}

/// Generate a empty maze.
pub fn generate_empty_maze(nb_column: u32, nb_line: u32) -> Result<Maze, ServerError> {
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

    Ok(maze)
}
