use std::collections::HashMap;

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
