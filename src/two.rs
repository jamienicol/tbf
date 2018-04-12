use std::path::Path;
use std::vec::Vec;

use cgmath;
use gfx;
use gfx::format::Formatted;
use gfx::handle;
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use gfx::traits::FactoryExt;
use image;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub type ColorFormat = gfx::format::Srgba8;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
        out: gfx::BlendTarget<ColorFormat> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub struct Texture<R>
where
    R: gfx::Resources,
{
    _texture: handle::Texture<R, <ColorFormat as Formatted>::Surface>,
    view: handle::ShaderResourceView<R, <ColorFormat as Formatted>::View>,
    sampler: handle::Sampler<R>,
}

impl<R> Texture<R>
where
    R: gfx::Resources,
{
    pub fn new<F>(factory: &mut F, path: &Path) -> Self
    where
        F: gfx::Factory<R>,
    {
        let img = image::open(path)
            .expect(&format!("Error opening image file {:?}", path))
            .to_rgba();
        let (w, h) = img.dimensions();
        let kind = gfx::texture::Kind::D2(w as u16, h as u16, gfx::texture::AaMode::Single);

        let (texture, view) = factory
            .create_texture_immutable_u8::<ColorFormat>(
                kind,
                gfx::texture::Mipmap::Provided,
                &[&img],
            )
            .expect("Error creating texture");

        let sampler_info = SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp);
        let sampler = factory.create_sampler(sampler_info);

        Self {
            _texture: texture,
            view,
            sampler,
        }
    }
}

pub struct Sprite {
    pub dest: Rect,
    pub src: Rect,
}

impl Sprite {
    pub fn new<F, R>(_factory: &mut F) -> Self
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
    {
        Self {
            dest: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            src: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        }
    }
}

pub struct SpriteBatch {
    pub sprites: Vec<Sprite>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }
}

pub struct Renderer<R>
where
    R: gfx::Resources,
{
    pso: gfx::PipelineState<R, pipe::Meta>,
}

impl<R> Renderer<R>
where
    R: gfx::Resources,
{
    pub fn new<F>(factory: &mut F) -> Renderer<R>
    where
        F: gfx::Factory<R>,
    {
        let pso = factory
            .create_pipeline_simple(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/resources/shaders/vert.glsl"
                )),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/resources/shaders/frag.glsl"
                )),
                pipe::new(),
            )
            .unwrap();

        Renderer { pso }
    }

    pub fn render_sprite<F, C>(
        &self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &handle::RenderTargetView<R, ColorFormat>,
        sprite: &Sprite,
        texture: &Texture<R>,
    ) where
        F: gfx::Factory<R>,
        C: gfx::CommandBuffer<R>,
    {
        let vertices: [Vertex; 6] = [
            Vertex {
                pos: [sprite.dest.x, sprite.dest.y],
                uv: [sprite.src.x, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.dest.x + sprite.dest.width, sprite.dest.y],
                uv: [sprite.src.x + sprite.src.width, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.dest.x, sprite.dest.y + sprite.dest.height],
                uv: [sprite.src.x, sprite.src.y + sprite.src.height],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.dest.x + sprite.dest.width, sprite.dest.y],
                uv: [sprite.src.x + sprite.src.width, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [
                    sprite.dest.x + sprite.dest.width,
                    sprite.dest.y + sprite.dest.height,
                ],
                uv: [
                    sprite.src.x + sprite.src.width,
                    sprite.src.y + sprite.src.height,
                ],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.dest.x, sprite.dest.y + sprite.dest.height],
                uv: [sprite.src.x, sprite.src.y + sprite.src.height],
                color: [1.0, 1.0, 1.0],
            },
        ];

        // TODO: creating a new buffer every draw can't be the most efficient
        let vbuf = factory
            .create_buffer(
                6,
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::empty(),
            )
            .expect("Failed to create vertex buffer");

        encoder.update_buffer(&vbuf, &vertices, 0).unwrap();

        let slice = gfx::Slice {
            start: 0,
            end: 6,
            base_vertex: 0,
            instances: None,
            buffer: gfx::IndexBuffer::Auto,
        };

        let data = pipe::Data {
            vbuf,
            texture: (texture.view.clone(), texture.sampler.clone()),
            proj: cgmath::ortho(0.0, 1280.0, 768.0, 0.0, 1.0, 0.0).into(),
            out: out.clone(),
        };

        encoder.draw(&slice, &self.pso, &data);
    }

    pub fn render_spritebatch<F, C>(
        &self,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>,
        out: &handle::RenderTargetView<R, ColorFormat>,
        spritebatch: &SpriteBatch,
        texture: &Texture<R>,
    ) where
        F: gfx::Factory<R>,
        C: gfx::CommandBuffer<R>,
    {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(spritebatch.sprites.len() * 6);
        for sprite in &spritebatch.sprites {
            vertices.push(Vertex {
                pos: [sprite.dest.x, sprite.dest.y],
                uv: [sprite.src.x, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                pos: [sprite.dest.x + sprite.dest.width, sprite.dest.y],
                uv: [sprite.src.x + sprite.src.width, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                pos: [sprite.dest.x, sprite.dest.y + sprite.dest.height],
                uv: [sprite.src.x, sprite.src.y + sprite.src.height],
                color: [1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                pos: [sprite.dest.x + sprite.dest.width, sprite.dest.y],
                uv: [sprite.src.x + sprite.src.width, sprite.src.y],
                color: [1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                pos: [
                    sprite.dest.x + sprite.dest.width,
                    sprite.dest.y + sprite.dest.height,
                ],
                uv: [
                    sprite.src.x + sprite.src.width,
                    sprite.src.y + sprite.src.height,
                ],
                color: [1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                pos: [sprite.dest.x, sprite.dest.y + sprite.dest.height],
                uv: [sprite.src.x, sprite.src.y + sprite.src.height],
                color: [1.0, 1.0, 1.0],
            });
        }

        // TODO: creating a new buffer every draw can't be the most efficient
        let vbuf = factory
            .create_buffer(
                vertices.len(),
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::empty(),
            )
            .expect("Failed to create vertex buffer");

        encoder.update_buffer(&vbuf, &vertices, 0).unwrap();

        let slice = gfx::Slice {
            start: 0,
            end: 6 * vertices.len() as u32,
            base_vertex: 0,
            instances: None,
            buffer: gfx::IndexBuffer::Auto,
        };

        let data = pipe::Data {
            vbuf,
            texture: (texture.view.clone(), texture.sampler.clone()),
            proj: cgmath::ortho(0.0, 1280.0, 768.0, 0.0, 1.0, 0.0).into(),
            out: out.clone(),
        };

        encoder.draw(&slice, &self.pso, &data);
    }
}
