use cgmath::Vector2;
use specs::{Fetch, Join, System, WriteStorage};

use components::{Cursor, CursorState, Movement, MovementStep, Position};
use resources::{DeltaTime, Input};

pub struct CursorSystem;

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        Fetch<'a, Input>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Movement>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, input, mut cursors, mut positions, mut movements) = data;

        let speed = 320.0;

        for (cursor, position, movement) in (&mut cursors, &mut positions, &mut movements).join() {
            // if we are not moving then we're allowed to accept a new input command
            let mut allow_input = movement.steps.is_empty();

            // if we're nearly finished moving then we're
            // allowed to accept a new input command
            if movement.steps.len() == 1 {
                let step = movement.steps.front().unwrap();
                let disp = step.target - position.pos;

                let required_dt_x = if step.velocity.x != 0.0 {
                    disp.x / step.velocity.x
                } else {
                    0.0
                };

                let required_dt_y = if step.velocity.y != 0.0 {
                    disp.y / step.velocity.y
                } else {
                    0.0
                };

                if required_dt_x <= dt.dt && required_dt_y <= dt.dt {
                    allow_input = true;
                }
            }

            if allow_input {
                let mut target = match movement.steps.back() {
                    Some(step) => step.target,
                    None => position.pos,
                };
                let mut velocity = Vector2::new(0.0, 0.0);

                if input.up {
                    target.y -= 64.0;
                    velocity.y -= speed;
                }
                if input.down {
                    target.y += 64.0;
                    velocity.y += speed;
                }
                if input.left {
                    target.x -= 64.0;
                    velocity.x -= speed;
                }
                if input.right {
                    target.x += 64.0;
                    velocity.x += speed;
                }

                if target != position.pos && velocity != Vector2::new(0.0, 0.0) {
                    cursor.state = CursorState::Moving;
                    movement.steps.push_back(MovementStep {
                        target: target,
                        velocity: velocity,
                    });
                }
            }
        }
    }
}
