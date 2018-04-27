use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;
use std::string::String;

use gfx;
use ggez::graphics::Image;
use specs::Entity;
use tiled;

use two;

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
        player: Entity,
    },
    SelectRun {
        player: Entity,
    },
    Running {
        player: Entity,
        // dest: Point2<f32>,
    },
    // SelectPass { player: Entity },
    // Passing { player: Entity, dest: Point2<f32> },
}

pub struct Turn {
    pub state: TurnState,
}
