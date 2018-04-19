use cgmath::Point2;
use gfx;
use gfx::handle::RenderTargetView;
use specs::{Fetch, Join, ReadStorage, System};

use components::{CanMove, Position, Size, Sprite};
use resources::{Assets, Map};

use two;

pub struct RenderSystem<'a, F, C, R>
where
    F: gfx::Factory<R> + 'a,
    C: gfx::CommandBuffer<R> + 'a,
    R: gfx::Resources + 'a,
{
    factory: &'a mut F,
    encoder: &'a mut gfx::Encoder<R, C>,
    out: &'a RenderTargetView<R, two::ColorFormat>,
    renderer: &'a two::Renderer<R>,
}

impl<'a, F, C, R> RenderSystem<'a, F, C, R>
where
    F: gfx::Factory<R>,
    C: gfx::CommandBuffer<R>,
    R: gfx::Resources,
{
    pub fn new(
        factory: &'a mut F,
        encoder: &'a mut gfx::Encoder<R, C>,
        out: &'a RenderTargetView<R, two::ColorFormat>,
        renderer: &'a two::Renderer<R>,
    ) -> Self {
        Self {
            factory,
            encoder,
            out,
            renderer,
        }
    }
}

impl<'a, 'b, F, C, R> System<'b> for RenderSystem<'a, F, C, R>
where
    F: gfx::Factory<R>,
    C: gfx::CommandBuffer<R>,
    R: gfx::Resources,
{
    type SystemData = (
        Fetch<'b, Assets<R>>,
        Fetch<'b, Map>,
        ReadStorage<'b, CanMove>,
        ReadStorage<'b, Position>,
        ReadStorage<'b, Size>,
        ReadStorage<'b, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, map, can_moves, positions, sizes, sprites) = data;

        // render map
        let tileset = &map.map.tilesets[0];
        let tileset_texture = &assets.images[&tileset.name];

        let mut spritebatch = two::SpriteBatch::new();

        let layer = &map.map.layers[0];
        for x in 0..map.map.width {
            for y in 0..map.map.height {
                let gid = layer.tiles[y as usize][x as usize];

                let tile_y = (gid - 1) / (tileset.images[0].width as u32 / tileset.tile_width);
                let tile_x = (gid - 1) % (tileset.images[0].width as u32 / tileset.tile_width);

                let mut sprite = two::Sprite::new(self.factory);

                sprite.dest = two::Rect {
                    x: (x * map.map.tile_width) as f32,
                    y: (y * map.map.tile_height) as f32,
                    width: tileset.tile_width as f32,
                    height: tileset.tile_height as f32,
                };

                sprite.src = two::Rect {
                    x: (tile_x * tileset.tile_width) as f32 / tileset.images[0].width as f32,
                    y: (tile_y * tileset.tile_height) as f32 / tileset.images[0].height as f32,
                    width: tileset.tile_width as f32 / tileset.images[0].width as f32,
                    height: tileset.tile_height as f32 / tileset.images[0].height as f32,
                };

                spritebatch.sprites.push(sprite);
            }
        }

        self.renderer.render_spritebatch(
            self.factory,
            self.encoder,
            &self.out,
            &spritebatch,
            tileset_texture,
        );

        // render highlights
        for (can_move, _position) in (&can_moves, &positions).join() {
            let mut highlights_batch = two::SpriteBatch::new();
            let mut lowlights_batch = two::SpriteBatch::new();

            for x in 0..map.map.width {
                for y in 0..map.map.height {
                    let mut sprite = two::Sprite::new(self.factory);
                    sprite.dest = two::Rect {
                        x: (x * map.map.tile_width) as f32,
                        y: (y * map.map.tile_height) as f32,
                        width: 64.0,
                        height: 64.0,
                    };
                    sprite.src = two::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 1.0,
                        height: 1.0,
                    };

                    if can_move.dests.contains(&Point2::new(x, y)) {
                        highlights_batch.sprites.push(sprite);
                    } else {
                        lowlights_batch.sprites.push(sprite);
                    }
                }
            }

            self.renderer.render_spritebatch(
                self.factory,
                self.encoder,
                &self.out,
                &highlights_batch,
                &assets.images["highlight"],
            );
            self.renderer.render_spritebatch(
                self.factory,
                self.encoder,
                &self.out,
                &lowlights_batch,
                &assets.images["lowlight"],
            );
        }

        // render sprite components
        for (position, size, sprite) in (&positions, &sizes, &sprites).join() {
            let texture = &assets.images[sprite.image_id];

            let mut sprite = two::Sprite::new(self.factory);
            sprite.dest = two::Rect {
                x: position.pos.x,
                y: position.pos.y,
                width: size.width,
                height: size.height,
            };
            sprite.src = two::Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            };

            self.renderer
                .render_sprite(self.factory, self.encoder, &self.out, &sprite, texture);
        }
    }
}
