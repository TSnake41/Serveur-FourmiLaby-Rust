//! Interface to an external maze generator.
mod cmaze;

use self::cmaze::CMaze;
use crate::{error::ServerError, maze::Maze};

/// Parameters needed by the maze generator.
#[repr(C)]
pub struct ParamMaze {
    pub nb_column: u32,
    pub nb_line: u32,
    pub nest_column: u32,
    pub nest_line: u32,
    pub nb_food: u32,
    pub difficulty: u32,
}

extern "C" {
    fn generateMaze(param: &ParamMaze) -> Option<&CMaze>;
    fn freeMaze(maze: &mut &CMaze);
}

/// Generate a [`Maze`] using the maze generator.
pub fn generate_maze(param: &ParamMaze) -> Result<Maze, ServerError> {
    // unsafe ok because generateMaze shouldn't crash, param is non-null.
    if let Some(mut cmaze) = unsafe { generateMaze(param) } {
        let maze: Result<Maze, _> = cmaze.try_into();

        // unsafe ok cmaze is non-null
        unsafe { freeMaze(&mut cmaze) }

        maze
    } else {
        Err(ServerError::Other(
            "External generator returned a null pointer !".into(),
        ))
    }
}
