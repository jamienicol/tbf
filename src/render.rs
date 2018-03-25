use ggez::Context;
use ggez::graphics::{DrawParam, Drawable, Point2, Rect};
use ggez::graphics::spritebatch::SpriteBatch;
use specs::{Fetch, Join, ReadStorage, System};

use components::{Position, Size, Sprite};
use resources::{Assets, Map};

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
        Fetch<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, map, positions, sizes, sprites) = data;

        let tileset = &map.map.tilesets[0];
        let tileset_image = assets.images[&tileset.name].clone();
        let mut spritebatch = SpriteBatch::new(tileset_image);

        let layer = &map.map.layers[0];
        for x in 0..map.map.width {
            for y in 0..map.map.height {
                let gid = layer.tiles[y as usize][x as usize];

                let tile_y = (gid - 1) / (tileset.images[0].width as u32 / tileset.tile_width);
                let tile_x = (gid - 1) % (tileset.images[0].width as u32 / tileset.tile_width);

                let src = Rect {
                    x: (tile_x * tileset.tile_width) as f32 / tileset.images[0].width as f32,
                    y: (tile_y * tileset.tile_height) as f32 / tileset.images[0].height as f32,
                    w: tileset.tile_width as f32 / tileset.images[0].width as f32,
                    h: tileset.tile_height as f32 / tileset.images[0].height as f32,
                };

                let dest = Point2::new(
                    (x * map.map.tile_width) as f32,
                    (y * map.map.tile_height) as f32,
                );

                spritebatch.add(DrawParam {
                    src: src,
                    dest: dest,
                    ..Default::default()
                });
            }
        }

        spritebatch
            .draw(&mut self.ctx, Point2::new(0.0, 0.0), 0.0)
            .unwrap();

        for (position, _size, sprite) in (&positions, &sizes, &sprites).join() {
            let image = &assets.images[sprite.image_id];
            image
                .draw(
                    &mut self.ctx,
                    Point2::new(position.pos.x.round(), position.pos.y.round()),
                    0.0,
                )
                .unwrap();
        }
    }
}
