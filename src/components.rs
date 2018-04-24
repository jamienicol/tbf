use cgmath::{Point2, Vector2};

#[derive(Component, Debug)]
pub struct TilePosition {
    pub pos: Point2<u32>,
}

#[derive(Component, Debug)]
pub struct SubTilePosition {
    pub pos: Point2<f32>,
}

#[derive(Component, Debug)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Debug)]
pub struct Sprite {
    pub image_id: &'static str,
}

#[derive(Debug)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Debug, PartialEq)]
pub enum CursorState {
    Still,
    Moving {
        velocity: Vector2<f32>,
        target: Point2<u32>,
    },
}

#[derive(Component, Debug)]
pub struct Cursor {
    pub state: CursorState,
}

pub enum PlayerState {
    Still,
    Running { path: Vec<Point2<u32>> },
}

#[derive(Component)]
pub struct Player {
    pub state: PlayerState,
}

#[derive(Component)]
pub struct CanMove {
    pub start: Point2<u32>,
    pub distance: u32,
    pub dests: Vec<Point2<u32>>,
    pub path: Vec<Point2<u32>>,
}
