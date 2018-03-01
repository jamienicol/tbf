use components::{Cursor, Position};
use resources::{DeltaTime, Input};
use specs::{Fetch, Join, ReadStorage, System, WriteStorage};

pub struct CursorSystem;

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        Fetch<'a, Input>,
        Fetch<'a, DeltaTime>,
        ReadStorage<'a, Cursor>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, dt, cursors, mut positions) = data;

        let speed = 320.0;

        for (_cursor, position) in (&cursors, &mut positions).join() {
            if input.up {
                position.y -= speed * dt.dt;
            }
            if input.down {
                position.y += speed * dt.dt;
            }
            if input.left {
                position.x -= speed * dt.dt;
            }
            if input.right {
                position.x += speed * dt.dt;
            }
        }
    }
}
