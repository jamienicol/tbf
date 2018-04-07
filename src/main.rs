extern crate cgmath;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

mod components;
mod cursor;
mod movement;
mod render;
mod resources;

use std::default::Default;
use std::path::Path;
use std::time;

use cgmath::Point2;
use specs::{RunNow, World};

use components::{Cursor, CursorState, Movement, Position, Size, Sprite};
use cursor::CursorSystem;
use movement::MovementSystem;
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input, Map};

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

mod sprite;

use gfx::Device;
use gfx::format::{DepthStencil, Rgba8};
use gfx::handle::RenderTargetView;
use glutin::{ContextBuilder, ElementState, Event, EventsLoop, GlContext, KeyboardInput,
             VirtualKeyCode, WindowBuilder, WindowEvent};

struct Game {
    world: World,
    cursor_system: CursorSystem,
    movement_system: MovementSystem,
}

impl Game {
    fn new<F, R>(factory: &mut F) -> Self
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
    {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Movement>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        let mut assets = Assets::new();

        assets.load_image(factory, "cursor.png", "cursor".to_string());

        // Load map
        let map =
            tiled::parse_file(Path::new("resources/pitch.tmx")).expect("Failed to parse map.");
        for tileset in &map.tilesets {
            assets.load_image(factory, &tileset.images[0].source, tileset.name.clone());
        }

        world.add_resource(DeltaTime { dt: 0.0 });
        world.add_resource(assets);
        world.add_resource(Input::default());
        world.add_resource(Map { map: map });

        // Create cursor
        world
            .create_entity()
            .with(Cursor {
                state: CursorState::Still,
            })
            .with(Position {
                pos: Point2::new(0.0, 0.0),
            })
            .with(Size {
                width: 64.0,
                height: 64.0,
            })
            .with(Movement::new())
            .with(Sprite { image_id: "cursor" })
            .build();

        let cursor_system = CursorSystem;
        let movement_system = MovementSystem;

        Self {
            world: world,
            cursor_system: cursor_system,
            movement_system: movement_system,
        }
    }

    fn on_keyboard_event(&mut self, event: &KeyboardInput) {
        let mut input = self.world.write_resource::<Input>();

        match event.virtual_keycode {
            Some(VirtualKeyCode::Left) => {
                input.left = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            Some(VirtualKeyCode::Up) => {
                input.up = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            Some(VirtualKeyCode::Right) => {
                input.right = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            Some(VirtualKeyCode::Down) => {
                input.down = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().dt = dt;

        self.cursor_system.run_now(&self.world.res);
        self.movement_system.run_now(&self.world.res);
    }

    fn render<F, R, C>(
        &mut self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &RenderTargetView<R, Rgba8>,
        sprite_renderer: &sprite::Renderer<R>,
    ) where
        F: gfx::Factory<R>,
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
    {
        let mut rs = RenderSystem::new(factory, encoder, out, sprite_renderer);
        rs.run_now(&self.world.res);
    }
}

fn main() {
    // Wayland backend has some issues, force X for now.
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Turn Based Football")
        .with_dimensions(1280, 800);
    let cb = ContextBuilder::new();
    let (window, mut device, mut factory, mut rtv, mut stv) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(wb, cb, &events_loop);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let renderer = sprite::Renderer::new(&mut factory);

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
                    WindowEvent::KeyboardInput { device_id, input } => {
                        game.on_keyboard_event(&input);
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
