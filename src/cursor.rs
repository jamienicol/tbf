use ggez::graphics::Vector2;
use specs::{Entities, Fetch, Join, System, WriteStorage};

use components::{Cursor, CursorState, Movement, Position};
use resources::Input;

pub struct CursorSystem;

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, Input>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Movement>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, input, mut cursors, mut positions, mut movements) = data;

        let speed = 320.0;

        for (entity, cursor, position) in (&*entities, &mut cursors, &mut positions).join() {
            // Check if we've finished moving
            if cursor.state == CursorState::Moving {
                let mut finished_moving = false;
                {
                    let movement = movements.get(entity).unwrap();
                    if position.pos == movement.target {
                        finished_moving = true;
                    }
                }
                if finished_moving {
                    movements.remove(entity);
                    cursor.state = CursorState::Still;
                }
            }

            // TODO: if we've nearly finished moving we should extend the target
            // so the movement is smooth

            // Check if we should start a movement
            if cursor.state == CursorState::Still {
                let mut target = position.pos;
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
                    let movement = Movement {
                        target: target,
                        velocity: velocity,
                    };
                    movements.insert(entity, movement);
                }
            }
        }
    }
}
