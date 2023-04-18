use std::{
    cell::RefMut,
    error::Error,
    net::{SocketAddr, TcpStream},
};

use fourmilaby_core::{
    client::{ClientGameView, ClientInstance},
    maze::Maze,
    message::types::{JoinMessageBody, Message, MoveDirection, MoveMessageBody},
};
use raylib::{self, ffi::KeyboardKey, prelude::*};

const TILE_WIDTH: u32 = 32;
const WALL_WIDTH: u32 = 8;

fn main() -> Result<(), Box<dyn Error>> {
    let arg: Option<String> = std::env::args().skip(1).next();

    if arg.is_none() {
        println!("usage: ./fourmilaby-client <IP>:<Port>");
        return Err("Not enough parameters".into());
    }

    let addr: SocketAddr = arg.unwrap_or_default().parse()?;
    let mut instance = ClientInstance::new(TcpStream::connect(addr)?);

    instance.join(JoinMessageBody {
        difficulty: 2,
        player_id: None,
    })?;

    instance.read_message()?;

    let maze = instance.view.maze.clone();

    let background_instance = instance.backgroundify()?;

    let (rl, thread) = raylib::init()
        .vsync()
        .size(1280, 720)
        .msaa_4x()
        .title("Client Fourmilaby")
        .build();

    while !rl.window_should_close() {
        let view = background_instance.view.lock().unwrap().clone();

        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            background_instance
                .sender
                .send(Message::Move(MoveMessageBody {
                    direction: MoveDirection::South,
                }))?;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            background_instance
                .sender
                .send(Message::Move(MoveMessageBody {
                    direction: MoveDirection::North,
                }))?;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            background_instance
                .sender
                .send(Message::Move(MoveMessageBody {
                    direction: MoveDirection::East,
                }))?;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            background_instance
                .sender
                .send(Message::Move(MoveMessageBody {
                    direction: MoveDirection::West,
                }))?;
        }

        rl.begin_drawing(&thread, |d| draw(d, view, &maze))
    }

    Ok(())
}

fn draw(d: RefMut<RaylibDrawHandle>, view: ClientGameView, maze: &Maze) {
    d.clear_background(Color::WHITE);

    for x in 0..maze.nb_column {
        for y in 0..maze.nb_line {
            let tile = maze
                .get_tile(x, y)
                .expect("Tried to render a incomplete maze");

            if let Some(pheromon_level) = view.pheromon.get((x + y * maze.nb_column) as usize) {
                d.draw_rectangle(
                    (x * TILE_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    TILE_WIDTH as i32,
                    TILE_WIDTH as i32,
                    Color::BLUE.fade(*pheromon_level),
                );
            }

            if tile.wall_north() {
                d.draw_rectangle(
                    (x * TILE_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    TILE_WIDTH as i32,
                    WALL_WIDTH as i32,
                    Color::BLACK,
                );
            }

            if tile.wall_south() {
                d.draw_rectangle(
                    (x * TILE_WIDTH) as i32,
                    ((y + 1) * TILE_WIDTH - WALL_WIDTH) as i32,
                    TILE_WIDTH as i32,
                    WALL_WIDTH as i32,
                    Color::BLACK,
                );
            }

            if tile.wall_west() {
                d.draw_rectangle(
                    (x * TILE_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    WALL_WIDTH as i32,
                    TILE_WIDTH as i32,
                    Color::BLACK,
                );
            }

            if tile.wall_east() {
                d.draw_rectangle(
                    ((x + 1) * TILE_WIDTH - WALL_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    WALL_WIDTH as i32,
                    TILE_WIDTH as i32,
                    Color::BLACK,
                );
            }

            if tile.is_food() {
                d.draw_text(
                    "F",
                    (x * TILE_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    TILE_WIDTH as i32,
                    Color::BLACK,
                );
            }

            if tile.is_nest() {
                d.draw_text(
                    "N",
                    (x * TILE_WIDTH) as i32,
                    (y * TILE_WIDTH) as i32,
                    TILE_WIDTH as i32,
                    Color::BLACK,
                );
            }
        }
    }

    d.draw_circle(
        (view.player_position.0 * TILE_WIDTH + TILE_WIDTH / 2) as i32,
        (view.player_position.1 * TILE_WIDTH + TILE_WIDTH / 2) as i32,
        TILE_WIDTH as f32 / 2.0,
        if view.player_has_food {
            Color::GREEN
        } else {
            Color::RED
        },
    );
}
