//! Maze representation and utilities.
pub mod generator;

use std::{
    boxed::Box,
    fmt::{Display, Write},
    mem,
    ops::Not,
};

use serde::{Deserialize, Serialize};

use crate::error::ServerError;

/// A wrapped [`Maze`] tile.
#[derive(Debug)]
#[repr(transparent)]
pub struct Tile(u8);

impl From<u8> for Tile {
    fn from(val: u8) -> Self {
        Tile(val)
    }
}

/// A maze or "Labyrinth".
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maze {
    pub nb_column: u32,
    pub nb_line: u32,
    pub nest_column: u32,
    pub nest_line: u32,

    /// Array of tiles, where each [`u8`] is an actual tile (that can be wrapped into a [`Tile`]).
    /// Is `nb_column` * `nb_line` large.
    ///
    /// Follows this ordering :
    /// ```text
    /// +-+-+-+ x ->
    /// |6|7|8|
    /// +-+-+-+
    /// |3|4|5|
    /// +-+-+-+
    /// |0|1|2|  ^
    /// +-+-+-+  | y
    ///
    /// index = x + y * nb_column
    /// ```
    pub tiles: Box<[u8]>,
}

/// Set the bit `n` for `x` with `value`.
fn set_bit(x: &mut u8, n: u8, value: bool) {
    if value {
        *x |= 1 << n;
    } else {
        *x &= (1u8 << n).not();
    }
}

impl Tile {
    pub fn wall_south(&self) -> bool {
        (self.0 & (1 << 0)) > 0
    }

    pub fn set_wall_south(&mut self, value: bool) {
        set_bit(&mut self.0, 0, value);
    }

    pub fn wall_west(&self) -> bool {
        (self.0 & (1 << 1)) > 0
    }

    pub fn set_wall_west(&mut self, value: bool) {
        set_bit(&mut self.0, 1, value);
    }

    pub fn wall_east(&self) -> bool {
        (self.0 & (1 << 2)) > 0
    }

    pub fn set_wall_east(&mut self, value: bool) {
        set_bit(&mut self.0, 2, value);
    }

    pub fn wall_north(&self) -> bool {
        (self.0 & (1 << 3)) > 0
    }

    pub fn set_wall_north(&mut self, value: bool) {
        set_bit(&mut self.0, 3, value);
    }

    pub fn is_nest(&self) -> bool {
        (self.0 & (1 << 4)) > 0
    }

    pub fn set_nest(&mut self, value: bool) {
        set_bit(&mut self.0, 4, value);
    }

    pub fn is_food(&self) -> bool {
        (self.0 & (1 << 5)) > 0
    }

    pub fn set_food(&mut self, value: bool) {
        set_bit(&mut self.0, 5, value);
    }
}

impl Maze {
    pub fn new(width: u32, height: u32, tiles: &[u8]) -> Result<Self, ServerError> {
        if (width * height) as usize != tiles.len() {
            return ServerError::invalid_maze_error(format!(
                "width * height doesn't match with tiles.len() ({width} * {height} != {})",
                tiles.len()
            ));
        }

        // Get the position of each nests.
        let nests: Vec<(u32, u32)> = tiles
            .iter()
            .enumerate() // get the index (=> position) of each tile
            .filter(|(_, tile)| Tile(**tile).is_nest())
            .map(|(i, _)| {
                (
                    /* x: */ (i as u32) % width,
                    /* y: */ (i as u32) / width,
                )
            })
            .collect();

        if nests.is_empty() {
            return ServerError::invalid_maze_error("No nest found in tiles");
        }

        if nests.len() > 1 {
            unimplemented!("Consider multiples nest");
        }

        let nest_pos = nests[0];

        Ok(Maze {
            nb_column: width,
            nb_line: height,
            nest_column: nest_pos.0,
            nest_line: nest_pos.1,
            tiles: tiles.into(),
        })
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<Tile> {
        if x >= self.nb_column || y >= self.nb_line {
            // Out of bounds
            None
        } else {
            // Get the tile, should exist.
            self.tiles
                .get((x + y * self.nb_line) as usize)
                .map(|tile| Tile(*tile))
        }
    }

    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x >= self.nb_column || y >= self.nb_line {
            // Out of bounds
            None
        } else {
            // Get the tile, should exist.
            self.tiles
                .get_mut((x + y * self.nb_line) as usize)
                .map(|x| unsafe {
                    // UNSAFE OK: Tile(u8) is repr(transparent)
                    mem::transmute(x)
                })
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile: &Tile) {
        if x >= self.nb_column || y >= self.nb_line {
            return;
        }

        // Get the tile, should exist.
        if let Some(x) = self.tiles.get_mut((x + y * self.nb_line) as usize) {
            *x = tile.0;
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::from("(Tile=");

        let _ = write!(buffer, "{};", self.0);
        _ = &buffer.write_char(if self.wall_north() { 'N' } else { ' ' });
        _ = &buffer.write_char(if self.wall_south() { 'S' } else { ' ' });
        _ = &buffer.write_char(if self.wall_west() { 'W' } else { ' ' });
        _ = &buffer.write_char(if self.wall_east() { 'E' } else { ' ' });
        _ = &buffer.write_str(if self.is_food() { ", Food" } else { "" });
        _ = &buffer.write_str(if self.is_nest() { ", Nest" } else { "" });

        _ = &buffer.write_char(')');

        write!(f, "{}", buffer)
    }
}
