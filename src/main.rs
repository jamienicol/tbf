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

    // // Wayland backend has some issues, force X for now.
    // std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    // let mut events_loop = EventsLoop::new();
    // let wb = WindowBuilder::new()
    //     .with_title("Turn Based Football")
    //     .with_dimensions(1280, 800);
    // let cb = ContextBuilder::new();
    // let (window, mut device, mut factory, mut rtv, mut stv) =
    //     gfx_window_glutin::init::<two::ColorFormat, DepthStencil>(wb, cb, &events_loop);

    // let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    // let sprite_renderer = two::Renderer::new(&mut factory);
    // let mut ui_renderer =
    //     conrod::backend::gfx::Renderer::new(&mut factory, &rtv, f64::from(window.hidpi_factor()))
    //         .unwrap();

    // let mut game = Game::new(&mut factory);

    // let mut prev_time = time::Instant::now();

    // let mut running = true;
    // 'main_loop: loop {
    //     events_loop.poll_events(|event| {
    //         if let Event::WindowEvent { event, .. } = event.clone() {
    //             match event {
    //                 WindowEvent::Closed => {
    //                     running = false;
    //                 }
    //                 WindowEvent::Resized(_, _) => {
    //                     gfx_window_glutin::update_views(&window, &mut rtv, &mut stv);
    //                 }
    //                 _ => (),
    //             }
    //         }

    //         game.on_event(&event);

    //         if let Some(input) =
    //             conrod::backend::winit::convert_event(event.clone(), window.window())
    //         {
    //             game.on_ui_input(input);
    //         }
    //     });

    //     if !running {
    //         break 'main_loop;
    //     }

    //     let new_time = time::Instant::now();
    //     let dt = new_time - prev_time;
    //     let dt_seconds = dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9;

    //     game.update(dt_seconds);

    //     prev_time = new_time;

    //     encoder.clear(&rtv, [1.0, 0.0, 0.0, 1.0]);

    //     game.render(
    //         &mut factory,
    //         &mut encoder,
    //         &rtv,
    //         &sprite_renderer,
    //         &mut ui_renderer,
    //     );

    //     encoder.flush(&mut device);
    //     window.swap_buffers().unwrap();
    //     device.cleanup();
    // }
}
