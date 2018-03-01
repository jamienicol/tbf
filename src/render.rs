use ggez::Context;
use ggez::graphics::{Drawable, Point2};
use specs::{Fetch, Join, ReadStorage, System};

use components::{Position, Size, Sprite};
use resources::Assets;

pub struct RenderSystem<'c> {
    ctx: &'c mut Context,
}

impl<'c> RenderSystem<'c> {
    pub fn new(ctx: &'c mut Context) -> Self {
        Self { ctx }
    }
}

impl<'a, 'c> System<'a> for RenderSystem<'c> {
    type SystemData = (
        Fetch<'a, Assets>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, positions, sizes, sprites) = data;

        for (position, _size, sprite) in (&positions, &sizes, &sprites).join() {
            let image = &assets.images[sprite.image_id];
            image
                .draw(
                    &mut self.ctx,
                    Point2::new(position.x.round(), position.y.round()),
                    0.0,
                )
                .unwrap();
        }
    }
}
