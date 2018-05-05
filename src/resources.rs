use std::collections::HashMap;
use std::default::Default;
use std::string::String;

use ggez::graphics::Image;
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
        }
    }
}

pub struct Map {
    pub map: tiled::Map,
}

#[derive(Debug, Clone)]
pub enum TurnState {
    SelectPlayer,
    ActionMenu {
        player_id: Entity,
    },
    SelectRun {
        player_id: Entity,
    },
    Running {
        player_id: Entity,
    },
    SelectPass {
        player_id: Entity,
        ball_id: Entity,
    },
    // Passing { player: Entity, dest: Point2<f32> },
}

pub struct Turn {
    pub state: TurnState,
}
