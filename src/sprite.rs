use cgmath;
use gfx;
use gfx::format::Srgba8;
use gfx::handle::{Buffer, RenderTargetView, Sampler, ShaderResourceView};
use gfx::traits::FactoryExt;
use gfx::{Encoder, Factory, PipelineState, Resources};

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
        out: gfx::BlendTarget<Srgba8> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub struct Sprite {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    tex_left: f32,
    tex_top: f32,
    tex_width: f32,
    tex_height: f32,
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            tex_left: 0.0,
            tex_top: 0.0,
            tex_width: 0.0,
            tex_height: 0.0,
        }
    }
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_tex_rect(&mut self, left: f32, top: f32, width: f32, height: f32) {
        self.tex_left = left;
        self.tex_top = top;
        self.tex_width = width;
        self.tex_height = height;
    }
}

pub struct Renderer<R>
where
    R: Resources,
{
    pso: PipelineState<R, pipe::Meta>,
    sampler: Sampler<R>,
    vbuf: Buffer<R, Vertex>,
}

impl<R> Renderer<R>
where
    R: Resources,
{
    pub fn new<F>(factory: &mut F) -> Renderer<R>
    where
        F: Factory<R>,
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

        let sampler = factory.create_sampler_linear();

        let vbuf = factory
            .create_buffer(
                6,
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::empty(),
            )
            .unwrap();

        Renderer {
            pso: pso,
            sampler: sampler,
            vbuf: vbuf,
        }
    }

    pub fn render_sprite<C>(
        &self,
        encoder: &mut Encoder<R, C>,
        out: &RenderTargetView<R, Srgba8>,
        sprite: &Sprite,
        texture: &ShaderResourceView<R, [f32; 4]>,
    ) where
        C: gfx::CommandBuffer<R>,
    {
        let vertices: [Vertex; 6] = [
            Vertex {
                pos: [sprite.x, sprite.y],
                uv: [sprite.tex_left, sprite.tex_top],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.x + sprite.width, sprite.y],
                uv: [sprite.tex_left + sprite.tex_width, sprite.tex_top],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.x, sprite.y + sprite.height],
                uv: [sprite.tex_left, sprite.tex_top + sprite.tex_height],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.x + sprite.width, sprite.y],
                uv: [sprite.tex_left + sprite.tex_width, sprite.tex_top],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.x + sprite.width, sprite.y + sprite.height],
                uv: [
                    sprite.tex_left + sprite.tex_width,
                    sprite.tex_top + sprite.tex_height,
                ],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [sprite.x, sprite.y + sprite.height],
                uv: [sprite.tex_left, sprite.tex_top + sprite.tex_height],
                color: [1.0, 1.0, 1.0],
            },
        ];

        encoder.update_buffer(&self.vbuf, &vertices, 0).unwrap();

        let slice = gfx::Slice {
            start: 0,
            end: 6,
            base_vertex: 0,
            instances: None,
            buffer: gfx::IndexBuffer::Auto,
        };

        let data = pipe::Data {
            vbuf: self.vbuf.clone(),
            texture: (texture.clone(), self.sampler.clone()),
            proj: cgmath::ortho(0.0, 1280.0, 768.0, 0.0, 1.0, 0.0).into(),
            out: out.clone(),
        };

        encoder.draw(&slice, &self.pso, &data);
    }
}
