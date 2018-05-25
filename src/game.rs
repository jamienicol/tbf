use std::path::Path;

use conrod::{self, Colorable, Positionable, Widget};
use gfx::handle::ShaderResourceView;
use gfx_device_gl;
use ggez::{event, graphics, timer, Context, GameResult};
use nalgebra::Point2;
use specs::{RunNow, World};
use tiled;

use components::{Ball, BallState, CanMove, Cursor, CursorState, Player, PlayerState, PlayerTeam,
                 Size, Sprite, SubTilePosition, TilePosition};
use ggez2conrod;
use render::RenderSystem;
use resources::{Assets, Camera, DeltaTime, Input, Map, Turn, TurnState};
use systems::{ActionMenuSystem, BallDribbleSystem, BallMovementSystem, CameraSystem,
              CursorMovementSystem, PassSelectSystem, PathSelectSystem, PlayerMovementSystem,
              PlayerSelectSystem, RunSelectSystem};

widget_ids!(pub struct WidgetIds {
    fps,
    turn_state,
    action_menu_run,
    action_menu_pass,
    action_menu_cancel,
});

fn create_cursor(world: &mut World, pos: &Point2<u32>) {
    world
        .create_entity()
        .with(Cursor {
            state: CursorState::Still,
        })
        .with(TilePosition { pos: *pos })
        .with(SubTilePosition {
            pos: Point2::new((pos.x * 64) as f32, (pos.y * 64) as f32),
        })
        .with(Size {
            width: 64.0,
            height: 64.0,
        })
        .with(Sprite { image_id: "cursor" })
        .build();
}

fn create_player(world: &mut World, pos: &Point2<u32>, team: PlayerTeam) {
    world
        .create_entity()
        .with(Player {
            state: PlayerState::Still,
            team,
        })
        .with(TilePosition { pos: *pos })
        .with(SubTilePosition {
            pos: Point2::new((pos.x * 64) as f32, (pos.y * 64) as f32),
        })
        .with(Size {
            width: 64.0,
            height: 64.0,
        })
        .with(Sprite {
            image_id: match team {
                PlayerTeam::Red => "player-red",
                PlayerTeam::Blue => "player-blue",
            },
        })
        .build();
}

fn create_ball(world: &mut World, pos: &Point2<u32>) {
    world
        .create_entity()
        .with(Ball {
            state: BallState::Free,
        })
        .with(TilePosition { pos: *pos })
        .with(SubTilePosition {
            pos: Point2::new((pos.x * 64) as f32, (pos.y * 64) as f32),
        })
        .with(Size {
            width: 64.0,
            height: 64.0,
        })
        .with(Sprite { image_id: "ball" })
        .build();
}

pub struct Game<'a> {
    world: World,
    camera_system: CameraSystem,
    cursor_movement_system: CursorMovementSystem,
    player_select_system: PlayerSelectSystem,
    run_select_system: RunSelectSystem,
    pass_select_system: PassSelectSystem,
    path_select_system: PathSelectSystem,
    player_movement_system: PlayerMovementSystem,
    ball_dribble_system: BallDribbleSystem,
    ball_movement_system: BallMovementSystem,

    ui_renderer: conrod::backend::gfx::Renderer<'a, gfx_device_gl::Resources>,
    ui: conrod::Ui,
    widget_ids: WidgetIds,
    ui_image_map: conrod::image::Map<(
        ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
        (u32, u32),
    )>,
}

impl<'a> Game<'a> {
    pub fn new(
        ctx: &mut Context,
        ui_renderer: conrod::backend::gfx::Renderer<'a, gfx_device_gl::Resources>,
    ) -> GameResult<Self> {
        let mut assets = Assets::new();

        let mut cursor_image = graphics::Image::new(ctx, "/cursor.png").unwrap();
        cursor_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("cursor".to_string(), cursor_image);
        let mut player_red_image = graphics::Image::new(ctx, "/player-red.png").unwrap();
        player_red_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("player-red".to_string(), player_red_image);
        let mut player_blue_image = graphics::Image::new(ctx, "/player-blue.png").unwrap();
        player_blue_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("player-blue".to_string(), player_blue_image);
        let mut highlight_image = graphics::Image::new(ctx, "/highlight.png").unwrap();
        highlight_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("highlight".to_string(), highlight_image);
        let mut path_image = graphics::Image::new(ctx, "/path.png").unwrap();
        path_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("path".to_string(), path_image);
        let mut ball_image = graphics::Image::new(ctx, "/ball.png").unwrap();
        ball_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("ball".to_string(), ball_image);

        // Load map
        let map =
            tiled::parse_file(Path::new("resources/pitch.tmx")).expect("Failed to parse map.");
        for tileset in &map.tilesets {
            let mut tileset_image =
                graphics::Image::new(ctx, format!("/{}", &tileset.images[0].source)).unwrap();
            tileset_image.set_filter(graphics::FilterMode::Nearest);
            assets.images.insert(tileset.name.clone(), tileset_image);
        }

        let mut world = World::new();
        world.register::<Ball>();
        world.register::<CanMove>();
        world.register::<Player>();
        world.register::<TilePosition>();
        world.register::<SubTilePosition>();
        world.register::<Size>();
        world.register::<Sprite>();
        world.register::<Cursor>();

        world.add_resource(assets);
        world.add_resource(Camera::new());
        world.add_resource(Map { map });
        world.add_resource(DeltaTime { dt: 0.0 });
        world.add_resource(Input::default());

        world.add_resource(Turn {
            state: TurnState::SelectPlayer,
        });

        create_cursor(&mut world, &Point2::new(0, 0));
        create_player(&mut world, &Point2::new(2, 2), PlayerTeam::Red);
        create_player(&mut world, &Point2::new(4, 4), PlayerTeam::Red);
        create_player(&mut world, &Point2::new(2, 6), PlayerTeam::Red);
        create_player(&mut world, &Point2::new(10, 2), PlayerTeam::Blue);
        create_player(&mut world, &Point2::new(12, 6), PlayerTeam::Blue);
        create_player(&mut world, &Point2::new(11, 8), PlayerTeam::Blue);

        create_ball(&mut world, &Point2::new(2, 4));

        let mut ui = conrod::UiBuilder::new([1280.0, 800.0]).build();
        let widget_ids = WidgetIds::new(ui.widget_id_generator());
        let ui_image_map = conrod::image::Map::new();

        const FONT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/DejaVuSans.ttf");
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        Ok(Self {
            world,
            camera_system: CameraSystem,
            cursor_movement_system: CursorMovementSystem,
            player_select_system: PlayerSelectSystem,
            run_select_system: RunSelectSystem,
            pass_select_system: PassSelectSystem,
            path_select_system: PathSelectSystem,
            player_movement_system: PlayerMovementSystem,
            ball_dribble_system: BallDribbleSystem,
            ball_movement_system: BallMovementSystem,

            ui_renderer,
            ui,
            widget_ids,
            ui_image_map,
        })
    }
}

impl<'a> event::EventHandler for Game<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::duration_to_f64(timer::get_delta(ctx));
        self.world.write_resource::<DeltaTime>().dt = dt as f32;

        self.camera_system.run_now(&self.world.res);

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
                self.ball_dribble_system.run_now(&self.world.res);
            }
            TurnState::SelectPass { .. } => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.path_select_system.run_now(&self.world.res);
                self.pass_select_system.run_now(&self.world.res);
            }
            TurnState::Passing { .. } => {
                self.ball_movement_system.run_now(&self.world.res);
            }
        }

        // Display frames per second in top left
        let fps = timer::get_fps(ctx);
        conrod::widget::Text::new(&format!("{:.0} FPS", fps))
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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        {
            let mut rs = RenderSystem::new(ctx);
            rs.run_now(&self.world.res);
        }

        {
            let (factory, _device, encoder, _dtv, _rtv) = graphics::get_gfx_objects(ctx);

            let primitives = self.ui.draw();
            self.ui_renderer
                .fill(encoder, (1280.0, 800.0), primitives, &self.ui_image_map);
            self.ui_renderer.draw(factory, encoder, &self.ui_image_map);
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
            event::Keycode::W => {
                input.w = true;
            }
            event::Keycode::A => {
                input.a = true;
            }
            event::Keycode::S => {
                input.s = true;
            }
            event::Keycode::D => {
                input.d = true;
            }
            _ => {}
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
            event::Keycode::W => {
                input.w = false;
            }
            event::Keycode::A => {
                input.a = false;
            }
            event::Keycode::S => {
                input.s = false;
            }
            event::Keycode::D => {
                input.d = false;
            }
            _ => {}
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        state: event::MouseState,
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    ) {
        let input = ggez2conrod::convert_mouse_motion_event(ctx, state, x, y, xrel, yrel);

        if let Some(input) = input {
            self.ui.handle_event(input);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let input = ggez2conrod::convert_mouse_button_down_event(ctx, button, x, y);

        if let Some(input) = input {
            self.ui.handle_event(input);
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let input = ggez2conrod::convert_mouse_button_up_event(ctx, button, x, y);

        if let Some(input) = input {
            self.ui.handle_event(input);
        }
    }
}
