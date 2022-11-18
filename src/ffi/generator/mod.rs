/// C representation of a maze, used by C API.
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
    fn generateMaze(param: *const ParamMaze) -> *const CMaze;
    fn freeMaze(maze: *mut *const CMaze);
}

/// Generate a [`Maze`] using the maze generator.
pub fn generate_maze(param: &ParamMaze) -> Result<Maze, ServerError> {
    // unsafe ok because generateMaze shouldn't crash, param is non-null.
    let mut cmaze_ptr = unsafe { generateMaze(param as *const ParamMaze) };

    if cmaze_ptr.is_null() {
        return Err(ServerError::Other(
            "External generator returned a null pointer !".into(),
        ));
    }

    // unsafe ok as cmaze_ptr is non-null
    let maze: Result<Maze, _> = (unsafe { &*cmaze_ptr }).try_into();

    // unsafe ok as cmaze_ptr is actually a pointer to a CMaze.
    unsafe { freeMaze(&mut cmaze_ptr as *mut *const CMaze) };

    maze
}
