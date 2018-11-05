extern crate gfx;
extern crate ggez;
extern crate image;
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

use ggez::conf::Conf;
use ggez::{event, graphics, Context};

use game::Game;

fn main() {
    let mut c = Conf::new();
    c.window_setup.title = "Turn Based Football".to_string();
    c.window_mode.width = 1280;
    c.window_mode.height = 800;
    let ctx = &mut Context::load_from_conf("tbf", "Jamie Nicol", c).unwrap();

    let game = &mut Game::new(ctx).unwrap();
    event::run(ctx, game).unwrap();
}
