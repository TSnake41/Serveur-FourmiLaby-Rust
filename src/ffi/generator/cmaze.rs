use crate::{error::ServerError, maze::Maze};

use core::slice;

/// A maze as represented as a C structure.
#[repr(C)]
pub struct CMaze {
    nb_column: u32,
    nb_line: u32,
    nest_column: u32,
    nest_line: u32,

    /// may be externally managed
    tiles: *const u8,
}

/**
Try converting a [`CMaze`] into a [`Maze`].

Fails if the maze is considered invalid by [`Maze::new`].
*/
impl TryInto<Maze> for &CMaze {
    type Error = ServerError;

    fn try_into(self) -> Result<Maze, ServerError> {
        Maze::new(self.nb_column, self.nb_line, unsafe {
            slice::from_raw_parts(
                self.tiles,
                (self.nb_column as usize) * (self.nb_line as usize),
            )
            //.into()
        })
    }
}

/// Try converting a CMaze into a Maze.
#[test]
fn cmaze_to_maze() {
    // This must outlive cmaze.
    let tiles = &[1u8 << 4, 3, 1, 2];

    let cmaze = CMaze {
        nb_column: 2,
        nb_line: 2,
        nest_column: 0,
        nest_line: 0,
        tiles: tiles.as_ptr(),
    };

    // Make a Maze from cmaze.
    let converted_maze: Maze = (&cmaze).try_into().unwrap();

    let expected_maze = Maze::new(2, 2, tiles).unwrap();

    // Check if mazes have the same tiles.
    assert!(&expected_maze.tiles.cmp(&converted_maze.tiles).is_eq());

    // Check if the mazes have the same properties.
    assert_eq!(expected_maze.nb_column, converted_maze.nb_column);
    assert_eq!(expected_maze.nb_line, converted_maze.nb_line);
    assert_eq!(expected_maze.nest_column, converted_maze.nest_column);
    assert_eq!(expected_maze.nest_line, converted_maze.nest_line);
}
