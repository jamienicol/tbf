use std::collections::VecDeque;

use cgmath::{Point2, Vector2};

#[derive(Component, Debug)]
pub struct Position {
    pub pos: Point2<f32>,
}

#[derive(Clone, Debug)]
pub struct MovementStep {
    pub target: Point2<f32>,
    pub velocity: Vector2<f32>,
}

#[derive(Component, Debug)]
pub struct Movement {
    pub steps: VecDeque<MovementStep>,
}

impl Movement {
    pub fn new() -> Self {
        Self {
            steps: VecDeque::new(),
        }
    }
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

#[derive(Debug, PartialEq)]
pub enum CursorState {
    Still,
    Moving,
}

#[derive(Component, Debug)]
pub struct Cursor {
    pub state: CursorState,
}
