use std::collections::HashMap;
use std::default::Default;

use ggez::graphics::Image;

pub struct Assets {
    pub images: HashMap<&'static str, Image>,
}

impl Assets {
    pub fn new() -> Self {
        let images = HashMap::new();

        Self { images: images }
    }
}

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