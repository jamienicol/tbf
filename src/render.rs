use ggez::graphics::spritebatch::SpriteBatch;
use ggez::{graphics, Context};
use nalgebra::Point2;
use specs::{Fetch, Join, ReadStorage, System};

use components::{CanMove, Direction, Size, Sprite, SubTilePosition};
use resources::{Assets, Camera, Map};

fn get_direction(from: &Point2<u32>, to: &Point2<u32>) -> Option<Direction> {
    if from.x < to.x && from.y == to.y {
        Some(Direction::Right)
    } else if from.x > to.x && from.y == to.y {
        Some(Direction::Left)
    } else if from.x == to.x && from.y < to.y {
        Some(Direction::Down)
    } else if from.x == to.x && from.y > to.y {
        Some(Direction::Up)
    } else {
        None
    }
}

fn get_path_directions(
    start: &Point2<u32>,
    path: &Vec<Point2<u32>>,
) -> Vec<(Direction, Option<Direction>)> {
    let mut directions = Vec::with_capacity(path.len());

    for (i, pos) in path.iter().enumerate() {
        let from = if i == 0 {
            get_direction(pos, start).unwrap()
        } else {
            get_direction(pos, &path[i - 1]).unwrap()
        };
        let to = if i == path.len() - 1 {
            None
        } else {
            get_direction(pos, &path[i + 1])
        };
        directions.push((from, to));
    }

    directions
}

fn get_path_draw_params(from: &Direction, to: &Option<Direction>) -> (u32, u32) {
    match (from, to) {
        (Direction::Left, Some(Direction::Right)) |
        (Direction::Right, Some(Direction::Left)) => (0, 0),
        (Direction::Up, Some(Direction::Down)) |
        (Direction::Down, Some(Direction::Up)) => (64, 0),
        (Direction::Left, None) => (128, 0),
        (Direction::Up, None) => (192, 0),
        (Direction::Right, None) => (128, 64),
        (Direction::Down, None) => (192, 64),
        (Direction::Left, Some(Direction::Up)) |
        (Direction::Up, Some(Direction::Left)) => (0, 64),
        (Direction::Up, Some(Direction::Right)) |
        (Direction::Right, Some(Direction::Up)) => (64, 64),
        (Direction::Left, Some(Direction::Down)) |
        (Direction::Down, Some(Direction::Left)) => (0, 128),
        (Direction::Down, Some(Direction::Right)) |
        (Direction::Right, Some(Direction::Down)) => (64, 128),
        _ => unreachable!()
    }
}

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
        Fetch<'b, Camera>,
        Fetch<'b, Map>,
        ReadStorage<'b, CanMove>,
        ReadStorage<'b, SubTilePosition>,
        ReadStorage<'b, Size>,
        ReadStorage<'b, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (assets, camera, map, can_moves, sub_tile_positions, _sizes, sprites) = data;

        graphics::set_transform(self.ctx, camera.mat);
        graphics::apply_transformations(self.ctx).unwrap();

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
        let path_image = &assets.images["path"];
        let mut path_batch = SpriteBatch::new(path_image.clone());
        for (can_move,) in (&can_moves,).join() {
            let directions = get_path_directions(&can_move.start, &can_move.path);
            for (path, direction) in can_move.path.iter().zip(directions.iter()) {
                let (src_x, src_y) = get_path_draw_params(&direction.0, &direction.1);
                let src = graphics::Rect {
                    x: src_x as f32 / path_image.width() as f32,
                    y: src_y as f32 / path_image.height() as f32,
                    w: 64.0 / path_image.width() as f32,
                    h: 64.0 / path_image.height() as f32,
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
