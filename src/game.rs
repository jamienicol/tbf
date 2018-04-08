use std::path::Path;

use cgmath::Point2;
use gfx;
use gfx::handle::RenderTargetView;
use gfx::format::Srgba8;
use glutin::{ElementState, KeyboardInput, VirtualKeyCode};
use specs::{RunNow, World};
use tiled;

use components::{Cursor, CursorState, Movement, Position, Size, Sprite};
use cursor::CursorSystem;
use movement::MovementSystem;
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input, Map};
use two;

pub struct Game {
    world: World,
    cursor_system: CursorSystem,
    movement_system: MovementSystem,
}

impl Game {
    pub fn new<F, R>(factory: &mut F) -> Self
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

    pub fn on_keyboard_event(&mut self, event: &KeyboardInput) {
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

    pub fn update(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().dt = dt;

        self.cursor_system.run_now(&self.world.res);
        self.movement_system.run_now(&self.world.res);
    }

    pub fn render<F, R, C>(
        &mut self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &RenderTargetView<R, Srgba8>,
        renderer: &two::Renderer<R>,
    ) where
        F: gfx::Factory<R>,
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
    {
        let mut rs = RenderSystem::new(factory, encoder, out, renderer);
        rs.run_now(&self.world.res);
    }
}
