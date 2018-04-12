use std::path::Path;

use cgmath::Point2;
use gfx;
use gfx::handle::RenderTargetView;
use glutin::{ElementState, KeyboardInput, VirtualKeyCode};
use specs::{RunNow, World};
use tiled;

use components::{Cursor, CursorState, Player, PlayerState, Position, Size, Sprite};
use systems::{ActionMenuSystem, CursorMovementSystem, PlayerSelectSystem, RunSelectSystem, PlayerMovementSystem};
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input, Map, Turn, TurnState};
use two;

pub struct Game {
    world: World,
    cursor_movement_system: CursorMovementSystem,
    player_select_system: PlayerSelectSystem,
    action_menu_system: ActionMenuSystem,
    run_select_system: RunSelectSystem,
    player_movement_system: PlayerMovementSystem,
}

impl Game {
    pub fn new<F, R>(factory: &mut F) -> Self
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
    {
        let mut world = World::new();
        world.register::<Player>();
        world.register::<Position>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        let mut assets = Assets::new();

        assets.load_image(factory, "cursor.png", "cursor".to_string());
        assets.load_image(factory, "player.png", "player".to_string());

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

        world.add_resource(Turn {
            state: TurnState::SelectPlayer,
        });

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
            .with(Sprite { image_id: "cursor" })
            .build();

        // Create players
        world
            .create_entity()
            .with(Player { state: PlayerState::Still })
            .with(Position {
                pos: Point2::new(128.0, 128.0),
            })
            .with(Size {
                width: 64.0,
                height: 64.0,
            })
            .with(Sprite { image_id: "player" })
            .build();

        world
            .create_entity()
            .with(Player { state: PlayerState::Still })
            .with(Position {
                pos: Point2::new(256.0, 256.0),
            })
            .with(Size {
                width: 64.0,
                height: 64.0,
            })
            .with(Sprite { image_id: "player" })
            .build();

        Self {
            world: world,
            cursor_movement_system: CursorMovementSystem,
            player_select_system: PlayerSelectSystem,
            action_menu_system: ActionMenuSystem,
            run_select_system: RunSelectSystem,
            player_movement_system: PlayerMovementSystem,
        }
    }

    pub fn on_focused_event(&mut self, focused: bool) {
        if focused == false {
            *self.world.write_resource::<Input>() = Input::default();
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
            Some(VirtualKeyCode::Space) => {
                input.select = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            Some(VirtualKeyCode::Escape) => {
                input.cancel = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().dt = dt;

        let state = self.world.read_resource::<Turn>().state.clone();
        match state {
            TurnState::SelectPlayer => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.player_select_system.run_now(&self.world.res);
            }
            TurnState::ActionMenu { .. } => {
                self.action_menu_system.run_now(&self.world.res);
            }
            TurnState::SelectRun { .. } => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.run_select_system.run_now(&self.world.res);
            }
            TurnState::Running { .. } => {
                self.player_movement_system.run_now(&self.world.res);
            }
        }

        // Reset input states which must be pressed each time rather than held
        let mut input = self.world.write_resource::<Input>();
        input.select = false;
        input.cancel = false;
    }

    pub fn render<F, R, C>(
        &mut self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &RenderTargetView<R, two::ColorFormat>,
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
