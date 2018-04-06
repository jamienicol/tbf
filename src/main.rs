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
use std::env;
use std::path::{Path, PathBuf};

use specs::{RunNow, World};

use components::{Cursor, CursorState, Movement, Position, Size, Sprite};
use cursor::CursorSystem;
use movement::MovementSystem;
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input, Map};

#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

mod sprite;

use gfx::Device;
use gfx::format::{DepthStencil, Rgba8};
use gfx::texture::Mipmap;
use glutin::{ContextBuilder, Event, EventsLoop, GlContext, WindowBuilder, WindowEvent};

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

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Movement>();
    world.register::<Size>();
    world.register::<Sprite>();
    world.register::<Cursor>();

    let mut assets = Assets::new();

    assets.load_image(&mut factory, "cursor.png", "cursor".to_string());

    // Load map
    let map = tiled::parse_file(Path::new("resources/pitch.tmx")).expect("Failed to parse map.");
    for tileset in &map.tilesets {
        assets.load_image(
            &mut factory,
            &tileset.images[0].source,
            tileset.name.clone(),
        );
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
                    _ => (),
                }
            }
        });

        if !running {
            break 'main_loop;
        }

        encoder.clear(&rtv, [1.0, 0.0, 0.0, 1.0]);

        {
            let mut rs = RenderSystem::new(&mut factory, &mut encoder, &rtv, &renderer);
            rs.run_now(&world.res);
        }

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

struct Game {
    world: World,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Movement>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        let s = Self { world: world };
        Ok(s)
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.world.write_resource::<DeltaTime>().dt =
            timer::duration_to_f64(timer::get_delta(ctx)) as f32;

        let mut cs = CursorSystem;
        cs.run_now(&self.world.res);

        let mut ms = MovementSystem;
        ms.run_now(&self.world.res);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        {
            // let mut rs = RenderSystem::new(&mut factory, &mut encoder, &rtv, &mut renderer);
            // rs.run_now(&self.world.res);
        }

        graphics::present(ctx);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        let mut input = self.world.write_resource::<Input>();

        match keycode {
            event::Keycode::Up => input.up = true,
            event::Keycode::Down => input.down = true,
            event::Keycode::Left => input.left = true,
            event::Keycode::Right => input.right = true,
            _ => (),
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        let mut input = self.world.write_resource::<Input>();

        match keycode {
            event::Keycode::Up => input.up = false,
            event::Keycode::Down => input.down = false,
            event::Keycode::Left => input.left = false,
            event::Keycode::Right => input.right = false,
            _ => (),
        }
    }
}
