extern crate ggez;
extern crate specs;
#[macro_use]
extern crate specs_derive;

mod assets;
mod components;

use std::default::Default;
use std::env;
use std::path::PathBuf;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics;
use ggez::graphics::{Drawable, Point2};
use specs::{Fetch, Join, ReadStorage, RunNow, System, World};

use assets::Assets;
use components::{Position, Size, Sprite};

struct RenderSystem<'c> {
    ctx: &'c mut Context,
}

impl<'c> RenderSystem<'c> {
    pub fn new(ctx: &'c mut Context) -> Self {
        Self { ctx }
    }
}

impl<'a, 'c> System<'a> for RenderSystem<'c> {
    type SystemData = (
        Fetch<'a, Assets>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, positions, sizes, sprites) = data;

        for (position, size, sprite) in (&positions, &sizes, &sprites).join() {
            let image = assets.images.get(sprite.image_id).unwrap();
            image
                .draw(&mut self.ctx, Point2::new(position.x, position.y), 0.0)
                .unwrap();
        }
    }
}

struct Game {
    world: World,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Size>();
        world.register::<Sprite>();

        let mut assets = Assets::new();

        let cursor_image = graphics::Image::new(ctx, "/cursor.png").unwrap();
        assets.images.insert("cursor", cursor_image);

        world.add_resource(assets);

        // Create cursor
        world
            .create_entity()
            .with(Position { x: 100.0, y: 200.0 })
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
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        {
            let mut rs = RenderSystem::new(ctx);
            rs.run_now(&mut self.world.res);
        }

        graphics::present(ctx);

        Ok(())
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
