use std::path::Path;

use cgmath::Point2;
use conrod::{self, Colorable, Positionable, Widget};
use fps_counter::FPSCounter;
use gfx;
use gfx_device_gl;
use gfx::handle::{RenderTargetView, ShaderResourceView};
use ggez::*;
use ggez::event;
use ggez::timer;
use ggez::graphics::{DrawMode};
use glutin::{ElementState, Event, VirtualKeyCode, WindowEvent};
use specs::{RunNow, World};
use tiled;

use components::{CanMove, Cursor, CursorState, Player, PlayerState, Size, Sprite, SubTilePosition,
                 TilePosition};
use render::RenderSystem;
use resources::{Assets, DeltaTime, Input, Map, Turn, TurnState};
use systems::{ActionMenuSystem, CursorMovementSystem, PathSelectSystem, PlayerMovementSystem,
              PlayerSelectSystem, RunSelectSystem};
use two;

widget_ids!(pub struct WidgetIds {
    fps,
    turn_state,
    action_menu_run,
    action_menu_pass,
    action_menu_cancel,
});

pub struct Game<R>
where
    R: gfx::Resources,
{
    world: World,
    cursor_movement_system: CursorMovementSystem,
    player_select_system: PlayerSelectSystem,
    run_select_system: RunSelectSystem,
    path_select_system: PathSelectSystem,
    player_movement_system: PlayerMovementSystem,

    fps_counter: FPSCounter,

    ui: conrod::Ui,
    widget_ids: WidgetIds,
    ui_image_map: conrod::image::Map<(ShaderResourceView<R, [f32; 4]>, (u32, u32))>,
}

impl<R> Game<R>
where
    R: gfx::Resources,
{
    pub fn new<F>(factory: &mut F) -> Self
    where
        F: gfx::Factory<R>,
    {
        let mut world = World::new();
        world.register::<CanMove>();
        world.register::<Player>();
        world.register::<TilePosition>();
        world.register::<SubTilePosition>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        let mut assets = Assets::new();

        assets.load_image(factory, "cursor.png", "cursor".to_string());
        assets.load_image(factory, "player.png", "player".to_string());
        assets.load_image(factory, "highlight.png", "highlight".to_string());
        assets.load_image(factory, "path.png", "path".to_string());

        // Load map
        let map =
            tiled::parse_file(Path::new("resources/pitch.tmx")).expect("Failed to parse map.");
        for tileset in &map.tilesets {
            assets.load_image(factory, &tileset.images[0].source, tileset.name.clone());
        }

        world.add_resource(DeltaTime { dt: 0.0 });
        world.add_resource(assets);
        world.add_resource(Input::default());
        world.add_resource(Map { map });

        world.add_resource(Turn {
            state: TurnState::SelectPlayer,
        });

        // Create cursor
        world
            .create_entity()
            .with(Cursor {
                state: CursorState::Still,
            })
            .with(TilePosition {
                pos: Point2::new(0, 0),
            })
            .with(SubTilePosition {
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
            .with(Player {
                state: PlayerState::Still,
            })
            .with(TilePosition {
                pos: Point2::new(2, 2),
            })
            .with(SubTilePosition {
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
            .with(Player {
                state: PlayerState::Still,
            })
            .with(TilePosition {
                pos: Point2::new(4, 4),
            })
            .with(SubTilePosition {
                pos: Point2::new(256.0, 256.0),
            })
            .with(Size {
                width: 64.0,
                height: 64.0,
            })
            .with(Sprite { image_id: "player" })
            .build();

        let fps_counter = FPSCounter::new();

        let mut ui = conrod::UiBuilder::new([1280.0, 800.0]).build();
        let widget_ids = WidgetIds::new(ui.widget_id_generator());
        let ui_image_map = conrod::image::Map::new();

        const FONT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/DejaVuSans.ttf");
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        Self {
            world,
            cursor_movement_system: CursorMovementSystem,
            player_select_system: PlayerSelectSystem,
            run_select_system: RunSelectSystem,
            path_select_system: PathSelectSystem,
            player_movement_system: PlayerMovementSystem,

            fps_counter,

            ui,
            widget_ids,
            ui_image_map,
        }
    }

    pub fn on_ui_input(&mut self, input: conrod::event::Input) {
        self.ui.handle_event(input);
    }

    pub fn update(&mut self, dt: f32) {
        self.world.write_resource::<DeltaTime>().dt = dt;

        let ui = &mut self.ui.set_widgets();

        let state = self.world.read_resource::<Turn>().state.clone();
        match state {
            TurnState::SelectPlayer => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.player_select_system.run_now(&self.world.res);
            }
            TurnState::ActionMenu { .. } => {
                let mut action_menu_system = ActionMenuSystem::new(ui, &self.widget_ids);
                action_menu_system.run_now(&self.world.res);
            }
            TurnState::SelectRun { .. } => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.path_select_system.run_now(&self.world.res);
                self.run_select_system.run_now(&self.world.res);
            }
            TurnState::Running { .. } => {
                self.player_movement_system.run_now(&self.world.res);
            }
        }

        // Display frames per second in top left
        let fps = self.fps_counter.tick();
        conrod::widget::Text::new(&format!("{} FPS", fps))
            .top_left_with_margin_on(ui.window, 8.0)
            .color(conrod::color::WHITE)
            .font_size(12)
            .set(self.widget_ids.fps, ui);

        // Display game state in bottom left
        conrod::widget::Text::new(&format!("{:?}", state))
            .bottom_left_with_margin_on(ui.window, 8.0)
            .color(conrod::color::WHITE)
            .font_size(12)
            .set(self.widget_ids.turn_state, ui);

        // Reset input states which must be pressed each time rather than held
        let mut input = self.world.write_resource::<Input>();
        input.select = false;
        input.cancel = false;
    }

    pub fn render<F, C>(
        &mut self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &RenderTargetView<R, two::ColorFormat>,
        sprite_renderer: &two::Renderer<R>,
        ui_renderer: &mut conrod::backend::gfx::Renderer<R>,
    ) where
        F: gfx::Factory<R>,
        C: gfx::CommandBuffer<R>,
    {
        {
            let mut rs = RenderSystem::new(factory, encoder, out, sprite_renderer);
            rs.run_now(&self.world.res);
        }

        let primitives = self.ui.draw();
        ui_renderer.fill(encoder, (1280.0, 800.0), primitives, &self.ui_image_map);
        ui_renderer.draw(factory, encoder, &self.ui_image_map);
    }
}

pub struct Game2<'a>
{
    game: Game<gfx_device_gl::Resources>,
    sprite_renderer: two::Renderer<gfx_device_gl::Resources>,
    ui_renderer: conrod::backend::gfx::Renderer<'a, gfx_device_gl::Resources>
}

impl<'a> Game2<'a>
{
    pub fn new(ctx: &mut Context,
               sprite_renderer: two::Renderer<gfx_device_gl::Resources>,
               ui_renderer: conrod::backend::gfx::Renderer<'a, gfx_device_gl::Resources>) -> GameResult<Self> {
        let factory = graphics::get_factory(ctx);
        let game = Game::new(factory);

        let s = Self {
            game,
            sprite_renderer,
            ui_renderer,
        };
        Ok(s)
    }
}

impl<'a> event::EventHandler for Game2<'a>
{
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::duration_to_f64(timer::get_delta(ctx));
        self.game.update(dt as f32);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        {
            let (factory, device, encoder, dtv, rtv) = graphics::get_gfx_objects(ctx);

            self.game.render(factory, encoder, &rtv, &self.sprite_renderer, &mut self.ui_renderer);
        }

        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool
    ) {
        let mut input = self.game.world.write_resource::<Input>();

        match keycode {
            event::Keycode::Left => {
                input.left = true;
            }
            event::Keycode::Up => {
                input.up = true;
            }
            event::Keycode::Down => {
                input.down = true;
            }
            event::Keycode::Right => {
                input.right = true;
            }
            event::Keycode::Return | event::Keycode::Space => {
                input.select = true;
            }
            event::Keycode::Escape => {
                input.cancel = true;
            }
            _ => {}
        }
    }

    fn key_up_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool
    ) {
        let mut input = self.game.world.write_resource::<Input>();

        match keycode {
            event::Keycode::Left => {
                input.left = false;
            }
            event::Keycode::Up => {
                input.up = false;
            }
            event::Keycode::Down => {
                input.down = false;
            }
            event::Keycode::Right => {
                input.right = false;
            }
            event::Keycode::Return | event::Keycode::Space => {
                input.select = false;
            }
            event::Keycode::Escape => {
                input.cancel = false;
            }
            _ => {}
        }
    }
}
