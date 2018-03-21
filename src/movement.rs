use ggez::graphics::Vector2;
use specs::{Fetch, Join, System, WriteStorage};

use components::{Movement, Position};
use resources::DeltaTime;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Movement>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut positions, mut movements) = data;

        for (position, movement) in (&mut positions, &mut movements).join() {
            let mut remaining_dt = dt.dt;

            while remaining_dt > 0.0 {
                if !movement.steps.is_empty() {
                    let step = movement.steps.front().unwrap().clone();
                    let disp = step.target - position.pos;

                    // calculate required time to travel remaining horizontal distance
                    let required_dt_x = if step.velocity.x != 0.0 {
                        remaining_dt.min(disp.x / step.velocity.x)
                    } else {
                        0.0
                    };

                    // calculate required time to travel remaining vertical distance
                    let required_dt_y = if step.velocity.y != 0.0 {
                        remaining_dt.min(disp.y / step.velocity.y)
                    } else {
                        0.0
                    };

                    // move and subtract used time from remaining time
                    position.pos += Vector2::new(
                        step.velocity.x * required_dt_x,
                        step.velocity.y * required_dt_y,
                    );
                    remaining_dt -= required_dt_x.max(required_dt_y);

                    // check if we've finished the step
                    if position.pos == step.target {
                        movement.steps.pop_front();
                    }
                } else {
                    break;
                }
            }
        }
    }
}
