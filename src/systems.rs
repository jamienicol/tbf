use cgmath::{InnerSpace, Point2, Vector2};
use conrod::{self, Labelable, Positionable, Sizeable, Widget};
use specs::{Entities, Fetch, FetchMut, Join, ReadStorage, System, WriteStorage};

use components::{CanMove, Cursor, CursorState, Direction, Player, PlayerState, Position};
use game::WidgetIds;
use resources::{DeltaTime, Input, Map, Turn, TurnState};

const CURSOR_SPEED: f32 = 320.0;
const PLAYER_SPEED: f32 = 640.0;

fn get_input(input: &Input) -> Option<Direction> {
    if input.left && !input.right {
        Some(Direction::Left)
    } else if input.up && !input.down {
        Some(Direction::Up)
    } else if input.right && !input.left {
        Some(Direction::Right)
    } else if input.down && !input.up {
        Some(Direction::Down)
    } else {
        None
    }
}

fn vector_from_direction(direction: &Direction) -> Vector2<f32> {
    match *direction {
        Direction::Left => Vector2::new(-1.0, 0.0),
        Direction::Up => Vector2::new(0.0, -1.0),
        Direction::Right => Vector2::new(1.0, 0.0),
        Direction::Down => Vector2::new(0.0, 1.0),
    }
}

fn required_time(displacement: f32, velocity: f32) -> f32 {
    if velocity != 0.0 {
        displacement / velocity
    } else {
        0.0
    }
}

pub struct CursorMovementSystem;

impl<'a> System<'a> for CursorMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        Fetch<'a, Input>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, input, mut cursors, mut positions) = data;

        for (cursor, position) in (&mut cursors, &mut positions).join() {
            let mut remaining_dt = dt.dt;

            while remaining_dt > 0.0 {
                if cursor.state == CursorState::Still {
                    if let Some(direction) = get_input(&input) {
                        let velocity = vector_from_direction(&direction) * CURSOR_SPEED;
                        let target = position.pos + vector_from_direction(&direction) * 64.0;

                        cursor.state = CursorState::Moving { velocity, target };
                    } else {
                        // Exit the loop as we will not use the remaining time to move.
                        break;
                    }
                }

                if let CursorState::Moving { velocity, target } = cursor.state {
                    let disp = target - position.pos;
                    let required_dt_x = required_time(disp.x, velocity.x);
                    let required_dt_y = required_time(disp.y, velocity.y);

                    let remaining_dt_x = remaining_dt.min(required_dt_x);
                    let remaining_dt_y = remaining_dt.min(required_dt_y);

                    position.pos +=
                        Vector2::new(velocity.x * remaining_dt_x, velocity.y * remaining_dt_y);
                    remaining_dt -= remaining_dt_x.max(remaining_dt_y);

                    if position.pos == target {
                        cursor.state = CursorState::Still;
                    }
                }
            }
        }
    }
}

pub struct PlayerSelectSystem;

impl<'a> System<'a> for PlayerSelectSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, Input>,
        FetchMut<'a, Turn>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, input, mut turn, cursors, positions, players) = data;

        for (_, cursor_pos) in (&cursors, &positions).join() {
            if input.select {
                for (entity, _, player_pos) in (&*entities, &players, &positions).join() {
                    if player_pos.pos == cursor_pos.pos {
                        turn.state = TurnState::ActionMenu { player: entity };
                        break;
                    }
                }
            }
        }
    }
}

pub struct ActionMenuSystem<'a, 'b>
where
    'b: 'a,
{
    ui: &'a mut conrod::UiCell<'b>,
    widget_ids: &'a WidgetIds,
}

impl<'a, 'b> ActionMenuSystem<'a, 'b> {
    pub fn new(ui: &'a mut conrod::UiCell<'b>, widget_ids: &'a WidgetIds) -> Self {
        ActionMenuSystem { ui, widget_ids }
    }
}

impl<'a, 'b, 'c> System<'c> for ActionMenuSystem<'a, 'b> {
    type SystemData = FetchMut<'c, Turn>;

    fn run(&mut self, data: Self::SystemData) {
        let mut turn = data;

        if let TurnState::ActionMenu { player } = turn.state {
            if conrod::widget::Button::new()
                .label("Run")
                .top_right_with_margin_on(self.ui.window, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_run, self.ui)
                .was_clicked()
            {
                turn.state = TurnState::SelectRun { player };
            }
            conrod::widget::Button::new()
                .label("Pass")
                .down_from(self.widget_ids.action_menu_run, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_pass, self.ui);
            if conrod::widget::Button::new()
                .label("Cancel")
                .down_from(self.widget_ids.action_menu_pass, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_cancel, self.ui)
                .was_clicked()
            {
                turn.state = TurnState::SelectPlayer;
            }
        }
    }
}

pub struct RunSelectSystem;

fn calculate_run_targets(
    start: Point2<u32>,
    map_size: Vector2<u32>,
    max_distance: u32,
) -> Vec<Point2<u32>> {
    let mut targets = Vec::new();

    // TODO make this based on actual pathfinding rather than just manhattan distance
    for x in 0..max_distance + 1 {
        for y in 0..max_distance - x + 1 {
            if start.x >= x {
                if start.y >= y {
                    targets.push(Point2::new(start.x - x, start.y - y));
                }
                if start.y < map_size.y - y {
                    targets.push(Point2::new(start.x - x, start.y + y));
                }
            }
            if start.x < map_size.x - x {
                if start.y >= y {
                    targets.push(Point2::new(start.x + x, start.y - y));
                }
                if start.y < map_size.y - y {
                    targets.push(Point2::new(start.x + x, start.y + y));
                }
            }
        }
    }

    return targets;
}

impl<'a> System<'a> for RunSelectSystem {
    type SystemData = (
        Fetch<'a, Input>,
        FetchMut<'a, Turn>,
        Fetch<'a, Map>,
        WriteStorage<'a, CanMove>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mut turn, map, mut can_moves, cursors, positions, mut players) = data;

        if let TurnState::SelectRun { player: player_ent } = turn.state {
            // Find the player
            let player = players.get_mut(player_ent).unwrap();
            let player_pos = positions.get(player_ent).unwrap();

            // Calculate where they can run to, if required
            if can_moves.get(player_ent).is_none() {
                let tile_pos = Point2::new(
                    (player_pos.pos.x / 64.0) as u32,
                    (player_pos.pos.y / 64.0) as u32,
                );
                let map_size = Vector2::new(map.map.width, map.map.height);
                let dests = calculate_run_targets(tile_pos, map_size, 8);
                let new_can_move = CanMove { dests };
                can_moves.insert(player_ent, new_can_move);
            }
            // FIXME: find a way to avoid this clone
            let move_dests = can_moves.get(player_ent).unwrap().dests.clone();

            for (cursor, cursor_pos) in (&cursors, &positions).join() {
                if input.select {
                    let cursor_tile_pos = Point2::new(
                        (cursor_pos.pos.x / 64.0) as u32,
                        (cursor_pos.pos.y / 64.0) as u32,
                    );
                    if cursor.state == CursorState::Still && cursor_pos.pos != player_pos.pos
                        && move_dests.contains(&cursor_tile_pos)
                    {
                        can_moves.remove(player_ent).unwrap();
                        turn.state = TurnState::Running {
                            player: player_ent,
                            dest: cursor_pos.pos,
                        };
                        player.state = PlayerState::Running {
                            velocity: (cursor_pos.pos - player_pos.pos).normalize_to(PLAYER_SPEED),
                            target: cursor_pos.pos,
                        }
                    }
                } else if input.cancel {
                    can_moves.remove(player_ent).unwrap();
                    turn.state = TurnState::SelectPlayer;
                }
            }
        }
    }
}

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        FetchMut<'a, Turn>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut turn, mut players, mut positions) = data;

        for (player, position) in (&mut players, &mut positions).join() {
            if let PlayerState::Running { velocity, target } = player.state {
                let disp = target - position.pos;
                let required_dt_x = required_time(disp.x, velocity.x);
                let required_dt_y = required_time(disp.y, velocity.y);

                let remaining_dt_x = dt.dt.min(required_dt_x);
                let remaining_dt_y = dt.dt.min(required_dt_y);

                position.pos +=
                    Vector2::new(velocity.x * remaining_dt_x, velocity.y * remaining_dt_y);

                if position.pos == target {
                    player.state = PlayerState::Still;
                    turn.state = TurnState::SelectPlayer;
                }
            }
        }
    }
}
