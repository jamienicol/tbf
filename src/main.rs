extern crate cgmath;
#[macro_use]
extern crate conrod;
extern crate fps_counter;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate ggez;
extern crate glutin;
extern crate image;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

mod components;
mod game;
mod render;
mod resources;
mod systems;
mod two;

use std::time;

use gfx::format::DepthStencil;
use gfx::Device;
use ggez::*;
use ggez::graphics::{DrawMode, Point2};
use glutin::{ContextBuilder, Event, EventsLoop, GlContext, WindowBuilder, WindowEvent};

use game::Game;
use game::Game2;

fn main() {
    let mut c = conf::Conf::new();
    c.window_setup.title = "Turn Based Football".to_string();
    c.window_mode.width = 1280;
    c.window_mode.height = 800;
    let ctx = &mut Context::load_from_conf("tbf", "Jamie Nicol", c).unwrap();

    let (sprite_renderer, ui_renderer) = {
        let (factory, device, encoder, dtv, rtv) = graphics::get_gfx_objects(ctx);

        let sprite_renderer = two::Renderer::new(factory);
        let mut ui_renderer =
            conrod::backend::gfx::Renderer::new(factory, &rtv, f64::from(1.0))
                .unwrap();

        (sprite_renderer, ui_renderer)
    };

    let game = &mut Game2::new(ctx, sprite_renderer, ui_renderer).unwrap();
    event::run(ctx, game).unwrap();
}
