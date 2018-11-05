extern crate ggez;
extern crate nalgebra;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

mod components;
mod game;
mod render;
mod resources;
mod systems;

use std::env;
use std::path::PathBuf;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::{event, ContextBuilder, GameResult};

use game::Game;

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let (ctx, events_loop) = &mut ContextBuilder::new("tbf", "Jamie Nicol")
        .window_setup(WindowSetup::default().title("Turn Based Football"))
        .window_mode(WindowMode::default().dimensions(1280.0, 800.0))
        .add_resource_path(resource_dir)
        .build()?;

    let game = &mut Game::new(ctx)?;
    event::run(ctx, events_loop, game)?;

    Ok(())
}
