#[macro_use]
extern crate conrod;
extern crate fps_counter;
extern crate gfx;
extern crate gfx_device_gl;
extern crate ggez;
extern crate image;
extern crate nalgebra;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

mod components;
mod game;
mod ggez2conrod;
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

    let ui_renderer = {
        let (factory, _device, _encoder, _dtv, rtv) = graphics::get_gfx_objects(ctx);

        conrod::backend::gfx::Renderer::new(factory, &rtv, f64::from(1.0)).unwrap()
    };

    let game = &mut Game::new(ctx, ui_renderer).unwrap();
    event::run(ctx, game).unwrap();
}
