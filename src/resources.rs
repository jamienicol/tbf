use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;
use std::string::String;

use gfx;
use gfx::format::Rgba8;
use gfx::handle::ShaderResourceView;
use gfx::texture::Mipmap;
use image;
use tiled;

pub struct Assets<R>
where
    R: gfx::Resources,
{
    pub images: HashMap<String, ShaderResourceView<R, [f32; 4]>>,
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
        let img = image::open(full_path)
            .expect(&format!("Error opening image file {}", &path))
            .to_rgba();
        let (w, h) = img.dimensions();
        let kind = gfx::texture::Kind::D2(w as u16, h as u16, gfx::texture::AaMode::Single);
        let (_, view) = factory
            .create_texture_immutable_u8::<Rgba8>(kind, Mipmap::Provided, &[&img])
            .expect(&format!("Error creating texture for image {}", &name));

        self.images.insert(name, view);
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
