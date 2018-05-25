use nalgebra::{Point2, Vector2};
use specs::Entity;

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

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
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

#[derive(Clone, Copy)]
pub enum PlayerTeam {
    Red,
    Blue,
}

#[derive(Component)]
pub struct Player {
    pub state: PlayerState,
    pub team: PlayerTeam,
}

#[derive(Component)]
pub struct CanMove {
    pub start: Point2<u32>,
    pub distance: u32,
    pub dests: Vec<Point2<u32>>,
    pub path: Vec<Point2<u32>>,
}

#[derive(PartialEq)]
pub enum BallState {
    Free,
    Possessed {
        player_id: Entity,
    },
    Moving {
        player_id: Entity,
        path: Vec<Point2<u32>>,
    },
}

#[derive(Component)]
pub struct Ball {
    pub state: BallState,
}
