use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;
use std::string::String;

use gfx;
use tiled;

use two;

pub struct Assets<R>
where
    R: gfx::Resources,
{
    pub images: HashMap<String, two::Texture<R>>,
}

impl<R> Assets<R>
where
    R: gfx::Resources,
{
    pub fn new() -> Self {
        let images = HashMap::new();

        Self { images: images }
    }

    pub fn load_image<F>(&mut self, factory: &mut F, path: &str, name: String)
    where
        F: gfx::Factory<R>,
    {
        let mut full_path = PathBuf::from("resources");
        full_path.push(&path);
        let texture = two::Texture::new(factory, &full_path);

        self.images.insert(name, texture);
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
