use std::collections::HashMap;
use std::default::Default;
use std::string::String;

use ggez::graphics::Image;
use nalgebra::{Matrix4, Vector3};
use specs::Entity;
use tiled;

pub struct Assets {
    pub images: HashMap<String, Image>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }
}

pub struct Camera {
    pub mat: Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            mat: Matrix4::new_translation(&Vector3::new(0.0, 0.0, 0.0)),
        }
    }
}

#[derive(Debug)]
pub struct DeltaTime {
    pub dt: f32,
}

#[derive(Debug)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub select: bool,
    pub cancel: bool,
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            select: false,
            cancel: false,
            w: false,
            a: false,
            s: false,
            d: false,
        }
    }
}

pub struct Map {
    pub map: tiled::Map,
}

#[derive(Debug, Clone)]
pub enum TurnState {
    SelectPlayer,
    ActionMenu { player_id: Entity },
    SelectRun { player_id: Entity },
    Running { player_id: Entity },
    SelectPass { player_id: Entity, ball_id: Entity },
    Passing { player_id: Entity, ball_id: Entity },
}

pub struct Turn {
    pub state: TurnState,
}
