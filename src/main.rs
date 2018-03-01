extern crate ggez;
extern crate specs;
#[macro_use]
extern crate specs_derive;

mod components;
mod cursor;
mod render;
mod resources;

use std::default::Default;
use std::env;
use std::path::PathBuf;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics;
use ggez::timer;
use specs::{RunNow, World};

use components::{Cursor, Position, Size, Sprite};
use cursor::CursorSystem;
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input};

struct Game {
    world: World,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        let mut assets = Assets::new();

        let cursor_image = graphics::Image::new(ctx, "/cursor.png").unwrap();
        assets.images.insert("cursor", cursor_image);

        world.add_resource(DeltaTime { dt: 0.0 });
        world.add_resource(assets);
        world.add_resource(Input::default());

        // Create cursor
        world
            .create_entity()
            .with(Cursor)
            .with(Position { x: 0.0, y: 0.0 })
            .with(Size {
                width: 64.0,
                height: 64.0,
            })
            .with(Sprite { image_id: "cursor" })
            .build();

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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        {
            let mut rs = RenderSystem::new(ctx);
            rs.run_now(&self.world.res);
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

fn main() {
    let mut cb = ContextBuilder::new("tbf", "Jamie Nicol");
    cb = cb.window_setup(WindowSetup {
        title: "Turn Based Football".to_string(),
        ..Default::default()
    });
    cb = cb.window_mode(WindowMode {
        width: 1280,
        height: 800,
        ..Default::default()
    });
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }
    let mut ctx = cb.build().unwrap();

    let mut state = Game::new(&mut ctx).unwrap();
    event::run(&mut ctx, &mut state).unwrap();
}
