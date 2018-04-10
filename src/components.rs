use cgmath::{Point2, Vector2};

#[derive(Component, Debug)]
pub struct Position {
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

#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Debug, PartialEq)]
pub enum CursorState {
    Still,
    Moving { velocity: Vector2<f32>, target: Point2<f32> },
}

#[derive(Component, Debug)]
pub struct Cursor {
    pub state: CursorState,
}
