mod grid;

use crate::{
    maze::Maze,
    message::types::{InfoMessageBody, MoveMessageBody},
};

use super::AntAI;

#[derive(Default)]
pub struct DfsAi {
    maze: Option<Maze>,
    grid: Option<grid::DfsPath>
}

impl AntAI for DfsAi {
    fn step(&mut self, maze: &Maze, message: &InfoMessageBody) -> Option<MoveMessageBody> {
        todo!()
    }
}
