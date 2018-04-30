use ggez::{graphics, Context};
use ggez::graphics::spritebatch::SpriteBatch;
use specs::{Fetch, Join, ReadStorage, System};

use components::{CanMove, Size, Sprite, SubTilePosition};
use resources::{Assets, Map};

pub struct RenderSystem<'a> {
    ctx: &'a mut Context,
}

impl<'a> RenderSystem<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }
}

impl<'a, 'b> System<'b> for RenderSystem<'a> {
    type SystemData = (
        Fetch<'b, Assets>,
        Fetch<'b, Map>,
        ReadStorage<'b, CanMove>,
        ReadStorage<'b, SubTilePosition>,
        ReadStorage<'b, Size>,
        ReadStorage<'b, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, map, can_moves, sub_tile_positions, _sizes, sprites) = data;

        // // render map
        let tileset = &map.map.tilesets[0];

        let mut tile_batch = SpriteBatch::new(assets.images[&tileset.name].clone());

        let layer = &map.map.layers[0];
        for x in 0..map.map.width {
            for y in 0..map.map.height {
                let gid = layer.tiles[y as usize][x as usize];

                let tile_y = (gid - 1) / (tileset.images[0].width as u32 / tileset.tile_width);
                let tile_x = (gid - 1) % (tileset.images[0].width as u32 / tileset.tile_width);

                let src = graphics::Rect {
                    x: (tile_x * tileset.tile_width) as f32 / tileset.images[0].width as f32,
                    y: (tile_y * tileset.tile_height) as f32 / tileset.images[0].height as f32,
                    w: tileset.tile_width as f32 / tileset.images[0].width as f32,
                    h: tileset.tile_height as f32 / tileset.images[0].height as f32,
                };
                let dest = graphics::Point2::new(
                    (x * map.map.tile_width) as f32,
                    (y * map.map.tile_height) as f32,
                );
                let param = graphics::DrawParam {
                    src,
                    dest,
                    ..graphics::DrawParam::default()
                };

                tile_batch.add(param);
            }
        }
        graphics::draw(self.ctx, &tile_batch, graphics::Point2::new(0.0, 0.0), 0.0).unwrap();

        // render highlights
        let mut highlight_batch = SpriteBatch::new(assets.images["highlight"].clone());
        for (can_move,) in (&can_moves,).join() {
            for dest in &can_move.dests {
                let src = graphics::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                };
                let dest = graphics::Point2::new(
                    (dest.x * map.map.tile_width) as f32,
                    (dest.y * map.map.tile_height) as f32,
                );
                let param = graphics::DrawParam {
                    src,
                    dest,
                    ..graphics::DrawParam::default()
                };

                highlight_batch.add(param);
            }
        }
        graphics::draw(
            self.ctx,
            &highlight_batch,
            graphics::Point2::new(0.0, 0.0),
            0.0,
        ).unwrap();

        // render paths
        let mut path_batch = SpriteBatch::new(assets.images["path"].clone());
        for (can_move,) in (&can_moves,).join() {
            for path in &can_move.path {
                let src = graphics::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                };
                let dest = graphics::Point2::new(
                    (path.x * map.map.tile_width) as f32,
                    (path.y * map.map.tile_height) as f32,
                );
                let param = graphics::DrawParam {
                    src,
                    dest,
                    ..graphics::DrawParam::default()
                };
                path_batch.add(param);
            }
        }
        graphics::draw(self.ctx, &path_batch, graphics::Point2::new(0.0, 0.0), 0.0).unwrap();

        // render sprite components
        for (position, sprite) in (&sub_tile_positions, &sprites).join() {
            let texture = &assets.images[sprite.image_id];

            let src = graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let dest = graphics::Point2::new(position.pos.x, position.pos.y);
            let param = graphics::DrawParam {
                src,
                dest,
                ..graphics::DrawParam::default()
            };

            graphics::draw_ex(self.ctx, texture, param).unwrap();
        }
    }
}
