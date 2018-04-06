use gfx;
use gfx::format::Rgba8;
use gfx::handle::RenderTargetView;
use specs::{Fetch, Join, ReadStorage, System};

use components::{Position, Size, Sprite};
use resources::{Assets, Map};

use sprite;

pub struct RenderSystem<'a, F, C, R>
where
    F: gfx::Factory<R> + 'a,
    C: gfx::CommandBuffer<R> + 'a,
    R: gfx::Resources + 'a,
{
    factory: &'a mut F,
    encoder: &'a mut gfx::Encoder<R, C>,
    out: &'a RenderTargetView<R, Rgba8>,
    sprite_renderer: &'a sprite::Renderer<R>,
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
        out: &'a RenderTargetView<R, Rgba8>,
        sprite_renderer: &'a sprite::Renderer<R>,
    ) -> Self {
        Self {
            factory: factory,
            encoder: encoder,
            out: out,
            sprite_renderer: sprite_renderer,
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
        ReadStorage<'b, Position>,
        ReadStorage<'b, Size>,
        ReadStorage<'b, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, map, positions, sizes, sprites) = data;

        // let tileset = &map.map.tilesets[0];
        // let tileset_image = assets.images[&tileset.name].clone();
        // let mut spritebatch = SpriteBatch::new(tileset_image);

        // let layer = &map.map.layers[0];
        // for x in 0..map.map.width {
        //     for y in 0..map.map.height {
        //         let gid = layer.tiles[y as usize][x as usize];

        //         let tile_y = (gid - 1) / (tileset.images[0].width as u32 / tileset.tile_width);
        //         let tile_x = (gid - 1) % (tileset.images[0].width as u32 / tileset.tile_width);

        //         let src = Rect {
        //             x: (tile_x * tileset.tile_width) as f32 / tileset.images[0].width as f32,
        //             y: (tile_y * tileset.tile_height) as f32 / tileset.images[0].height as f32,
        //             w: tileset.tile_width as f32 / tileset.images[0].width as f32,
        //             h: tileset.tile_height as f32 / tileset.images[0].height as f32,
        //         };

        //         let dest = Point2::new(
        //             (x * map.map.tile_width) as f32,
        //             (y * map.map.tile_height) as f32,
        //         );

        //         spritebatch.add(DrawParam {
        //             src: src,
        //             dest: dest,
        //             ..Default::default()
        //         });
        //     }
        // }

        // spritebatch
        //     .draw(&mut self.ctx, Point2::new(0.0, 0.0), 0.0)
        //     .unwrap();

        // for (position, _size, sprite) in (&positions, &sizes, &sprites).join() {
        //     let image = &assets.images[sprite.image_id];
        //     image
        //         .draw(
        //             &mut self.ctx,
        //             Point2::new(position.pos.x.round(), position.pos.y.round()),
        //             0.0,
        //         )
        //         .unwrap();
        // }

        for (position, size, sprite) in (&positions, &sizes, &sprites).join() {
            let texture = &assets.images[sprite.image_id];

            let mut sprite = sprite::Sprite::new();
            sprite.set_pos(position.pos.x, position.pos.y);
            sprite.set_size(size.width, size.height);
            sprite.set_tex_rect(0.0, 0.0, 1.0, 1.0);

            self.sprite_renderer
                .render_sprite(self.encoder, &self.out, &sprite, texture);
        }
    }
}
