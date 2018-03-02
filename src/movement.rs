use ggez::graphics::Vector2;
use specs::{Fetch, Join, ReadStorage, System, WriteStorage};

use components::{Movement, Position};
use resources::DeltaTime;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Movement>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut positions, movements) = data;

        for (position, movement) in (&mut positions, &movements).join() {
            let disp = movement.velocity * dt.dt;
            let remaining = movement.target - position.pos;
            let actual = Vector2::new(
                disp.x.abs().min(remaining.x.abs()) * disp.x.signum(),
                disp.y.abs().min(remaining.y.abs()) * disp.y.signum(),
            );
            position.pos += actual;
        }
    }
}
