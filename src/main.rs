extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
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

use gfx::Device;
use gfx::format::DepthStencil;
use glutin::{ContextBuilder, Event, EventsLoop, GlContext, WindowBuilder, WindowEvent};

use game::Game;

fn main() {
    // Wayland backend has some issues, force X for now.
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Turn Based Football")
        .with_dimensions(1280, 800);
    let cb = ContextBuilder::new();
    let (window, mut device, mut factory, mut rtv, mut stv) =
        gfx_window_glutin::init::<two::ColorFormat, DepthStencil>(wb, cb, &events_loop);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let renderer = two::Renderer::new(&mut factory);

    let mut game = Game::new(&mut factory);

    let mut prev_time = time::Instant::now();

    let mut running = true;
    'main_loop: loop {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Closed => {
                        running = false;
                    }
                    WindowEvent::Resized(_, _) => {
                        gfx_window_glutin::update_views(&window, &mut rtv, &mut stv);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        game.on_keyboard_event(&input);
                    }
                    WindowEvent::Focused(focused) => {
                        game.on_focused_event(focused);
                    }
                    _ => (),
                }
            }
        });

        if !running {
            break 'main_loop;
        }

        let new_time = time::Instant::now();
        let dt = new_time - prev_time;
        let dt_seconds = dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9;

        game.update(dt_seconds);

        prev_time = new_time;

        encoder.clear(&rtv, [1.0, 0.0, 0.0, 1.0]);

        game.render(&mut factory, &mut encoder, &rtv, &renderer);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
