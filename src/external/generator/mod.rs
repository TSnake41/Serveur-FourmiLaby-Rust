//! Interface to an external maze generator.
mod cmaze;

use self::cmaze::CMaze;
use crate::{
    config::{GeneratorConfig, NestPositioning},
    error::ServerError,
    maze::Maze,
    message::types::JoinMessageBody,
};

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
pub fn generate_maze(
    critera: &JoinMessageBody,
    config: &GeneratorConfig,
    rng: &fastrand::Rng,
) -> Result<Maze, ServerError> {
    let nb_column = config.column_min + (config.column_coeff * critera.difficulty as f32) as u32;
    let nb_line = config.line_min + (config.line_coeff * critera.difficulty as f32) as u32;

    let (nest_column, nest_line) = match config.nest_pos {
        NestPositioning::Randomized => (rng.u32(0..nb_column), rng.u32(0..nb_line)),
        NestPositioning::Fixed(x, y) => (x, y),
    };

    let param = ParamMaze {
        nb_column,
        nb_line,
        nest_column,
        nest_line,
        nb_food: config.nb_food_min + (config.nb_food_coeff * critera.difficulty as f32) as u32,
        difficulty: critera.difficulty,
    };

    // unsafe ok because generateMaze shouldn't crash, param is non-null.
    if let Some(mut cmaze) = unsafe { generateMaze(&param) } {
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
