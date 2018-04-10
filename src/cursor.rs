use cgmath::Vector2;
use specs::{Entities, Fetch, FetchMut, Join, ReadStorage, System, WriteStorage};

use components::{Cursor, CursorState, Direction, Player, Position};
use resources::{DeltaTime, Input, Turn, TurnState};

pub struct CursorMovementSystem;

fn get_input(input: &Input) -> Option<Direction> {
    if input.left && !input.right {
        return Some(Direction::Left);
    } else if input.up && !input.down {
        return Some(Direction::Up);
    } else if input.right && !input.left {
        return Some(Direction::Right);
    } else if input.down && !input.up {
        return Some(Direction::Down);
    } else {
        return None;
    }
}

fn vector_from_direction(direction: Direction) -> Vector2<f32> {
    return match direction {
        Direction::Left => Vector2::new(-1.0, 0.0),
        Direction::Up => Vector2::new(0.0, -1.0),
        Direction::Right => Vector2::new(1.0, 0.0),
        Direction::Down => Vector2::new(0.0, 1.0),
    };
}

fn required_time(displacement: f32, velocity: f32) -> f32 {
    if velocity != 0.0 {
        displacement / velocity
    } else {
        0.0
    }
}

impl<'a> System<'a> for CursorMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        Fetch<'a, Input>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, input, mut cursors, mut positions) = data;

        let speed = 320.0;

        for (cursor, position) in (&mut cursors, &mut positions).join() {
            let mut remaining_dt = dt.dt;

            while remaining_dt > 0.0 {
                if cursor.state == CursorState::Still {
                    if let Some(direction) = get_input(&input) {
                        let velocity = vector_from_direction(direction.clone()) * speed;
                        let target = position.pos + vector_from_direction(direction.clone()) * 64.0;

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

                    position.pos += Vector2::new(
                        velocity.x * remaining_dt_x,
                        velocity.y * remaining_dt_y,
                    );
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

        for (cursor, cursor_pos) in (&cursors, &positions).join() {
            if input.select {
                for (entity, player, player_pos) in (&*entities, &players, &positions).join() {
                    if player_pos.pos == cursor_pos.pos {
                        turn.state = TurnState::ActionMenu { player: entity };
                        break;
                    }
                }
            }
        }
    }
}
