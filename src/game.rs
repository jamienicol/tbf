use std::path::Path;

use ggez::{event, graphics, timer, Context, GameResult};
use nalgebra::Point2;
use specs::{RunNow, World};
use tiled;

use components::{
    Ball, BallState, CanMove, Cursor, CursorState, Player, PlayerState, PlayerTeam, Size, Sprite,
    SubTilePosition, TilePosition,
};
use render::RenderSystem;
use resources::{Assets, Camera, DeltaTime, Input, Map, Turn, TurnState};
use systems::{
    ActionMenuSystem, BallDribbleSystem, BallMovementSystem, CameraSystem, CursorMovementSystem,
    PassSelectSystem, PathSelectSystem, PlayerMovementSystem, PlayerSelectSystem, RunSelectSystem,
};

fn create_cursor(world: &mut World, pos: Point2<u32>) {
    world
        .create_entity()
        .with(Cursor {
            state: CursorState::Still,
        })
        .with(TilePosition { pos })
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

fn create_player(world: &mut World, pos: Point2<u32>, team: PlayerTeam) {
    world
        .create_entity()
        .with(Player {
            state: PlayerState::Still,
            team,
        })
        .with(TilePosition { pos })
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

fn create_ball(world: &mut World, pos: Point2<u32>) {
    world
        .create_entity()
        .with(Ball {
            state: BallState::Free,
        })
        .with(TilePosition { pos })
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

pub struct Game {
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
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut assets = Assets::new();

        let mut cursor_image = graphics::Image::new(ctx, "/cursor.png").unwrap();
        cursor_image.set_filter(graphics::FilterMode::Nearest);
        assets.images.insert("cursor".to_string(), cursor_image);
        let mut player_red_image = graphics::Image::new(ctx, "/player-red.png").unwrap();
        player_red_image.set_filter(graphics::FilterMode::Nearest);
        assets
            .images
            .insert("player-red".to_string(), player_red_image);
        let mut player_blue_image = graphics::Image::new(ctx, "/player-blue.png").unwrap();
        player_blue_image.set_filter(graphics::FilterMode::Nearest);
        assets
            .images
            .insert("player-blue".to_string(), player_blue_image);
        let mut highlight_image = graphics::Image::new(ctx, "/highlight.png").unwrap();
        highlight_image.set_filter(graphics::FilterMode::Nearest);
        assets
            .images
            .insert("highlight".to_string(), highlight_image);
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

        create_cursor(&mut world, Point2::new(0, 0));
        create_player(&mut world, Point2::new(2, 2), PlayerTeam::Red);
        create_player(&mut world, Point2::new(4, 4), PlayerTeam::Red);
        create_player(&mut world, Point2::new(2, 6), PlayerTeam::Red);
        create_player(&mut world, Point2::new(10, 2), PlayerTeam::Blue);
        create_player(&mut world, Point2::new(12, 6), PlayerTeam::Blue);
        create_player(&mut world, Point2::new(11, 8), PlayerTeam::Blue);

        create_ball(&mut world, Point2::new(2, 4));

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
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::duration_to_f64(timer::delta(ctx));
        self.world.write_resource::<DeltaTime>().dt = dt as f32;

        self.camera_system.run_now(&self.world.res);

        let state = self.world.read_resource::<Turn>().state.clone();
        match state {
            TurnState::SelectPlayer => {
                self.cursor_movement_system.run_now(&self.world.res);
                self.player_select_system.run_now(&self.world.res);
            }
            TurnState::ActionMenu { .. } => {
                let mut action_menu_system = ActionMenuSystem;
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

        // Reset input states which must be pressed each time rather than held
        let mut input = self.world.write_resource::<Input>();
        input.select = false;
        input.cancel = false;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));

        {
            let mut rs = RenderSystem::new(ctx);
            rs.run_now(&self.world.res);
        }

        // Display frames per second in top left
        let mut fps_text = graphics::Text::new(format!("{:.0} FPS", timer::fps(ctx)));
        fps_text.set_font(graphics::Font::default(), graphics::Scale::uniform(24.0));
        graphics::draw(ctx, &fps_text, (Point2::new(8.0, 8.0), graphics::WHITE))?;

        // Display the game state in bottom left
        let state = self.world.read_resource::<Turn>().state.clone();
        let mut state_text = graphics::Text::new(format!("{:?}", state));
        state_text.set_font(graphics::Font::default(), graphics::Scale::uniform(24.0));
        let height = state_text.height(ctx) as f32;
        graphics::draw(
            ctx,
            &state_text,
            (Point2::new(8.0, 800.0 - height - 8.0), graphics::WHITE),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        let mut input = self.world.write_resource::<Input>();

        match keycode {
            event::KeyCode::Left => {
                input.left = true;
            }
            event::KeyCode::Up => {
                input.up = true;
            }
            event::KeyCode::Down => {
                input.down = true;
            }
            event::KeyCode::Right => {
                input.right = true;
            }
            event::KeyCode::Return | event::KeyCode::Space => {
                input.select = true;
            }
            event::KeyCode::Escape => {
                input.cancel = true;
            }
            event::KeyCode::W => {
                input.w = true;
            }
            event::KeyCode::A => {
                input.a = true;
            }
            event::KeyCode::S => {
                input.s = true;
            }
            event::KeyCode::D => {
                input.d = true;
            }
            _ => {}
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
    ) {
        let mut input = self.world.write_resource::<Input>();

        match keycode {
            event::KeyCode::Left => {
                input.left = false;
            }
            event::KeyCode::Up => {
                input.up = false;
            }
            event::KeyCode::Down => {
                input.down = false;
            }
            event::KeyCode::Right => {
                input.right = false;
            }
            event::KeyCode::Return | event::KeyCode::Space => {
                input.select = false;
            }
            event::KeyCode::Escape => {
                input.cancel = false;
            }
            event::KeyCode::W => {
                input.w = false;
            }
            event::KeyCode::A => {
                input.a = false;
            }
            event::KeyCode::S => {
                input.s = false;
            }
            event::KeyCode::D => {
                input.d = false;
            }
            _ => {}
        }
    }
}
