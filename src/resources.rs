use std::collections::HashMap;
use std::default::Default;
use std::string::String;

use ggez::graphics::Image;
use tiled;

pub struct Assets {
    pub images: HashMap<String, Image>,
}

impl Assets {
    pub fn new() -> Self {
        let images = HashMap::new();

        Self { images: images }
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
}

impl Default for Input {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

pub struct Map {
    pub map: tiled::Map,
}
