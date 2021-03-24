use glium::{buffer::WriteMapping, implement_vertex, uniform, Surface};

use crate::{
    shader_program,
    world::{self, WorldPosition},
};

use super::{Pass, PassGroup, SurfaceProvider};

#[derive(Copy, Clone)]
struct FaceInfo {
    position: [f32; 3],
    color: [f32; 3],
    face: u32,
}

implement_vertex!(FaceInfo, position, color, face);

#[derive(Copy, Clone)]
struct Projection {
    perspective: glam::Mat4,
    view_model: glam::Mat4,
}

impl Projection {
    fn new(aspect_ratio: f32, eye: glam::Vec3, center: glam::Vec3) -> Self {
        let perspective =
            glam::Mat4::perspective_rh_gl(f32::to_radians(90.0), aspect_ratio, 0.1, 1024.0);
        let view_model = glam::Mat4::look_at_rh(eye, center, glam::vec3(0.0, 1.0, 0.0));
        Self {
            perspective,
            view_model,
        }
    }

    fn to_uniform(self) -> impl glium::uniforms::Uniforms {
        uniform! {
            perspective: self.perspective.to_cols_array_2d(),
            view_model: self.view_model.to_cols_array_2d(),
        }
    }
}

const CUBES: [FaceInfo; 12] = [
    FaceInfo {
        position: [-1.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 3,
    },
    FaceInfo {
        position: [-1.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 1,
    },
    FaceInfo {
        position: [-1.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 5,
    },
    FaceInfo {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 3,
    },
    FaceInfo {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 0.0, 1.0],
        face: 1,
    },
    FaceInfo {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 5,
    },
    FaceInfo {
        position: [0.0, -1.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 3,
    },
    FaceInfo {
        position: [0.0, -1.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 1,
    },
    FaceInfo {
        position: [0.0, -1.0, 0.0],
        color: [0.0, 1.0, 0.0],
        face: 5,
    },
    FaceInfo {
        position: [0.0, -1.0, -1.0],
        color: [0.0, 1.0, 0.0],
        face: 3,
    },
    FaceInfo {
        position: [0.0, -1.0, -1.0],
        color: [0.0, 1.0, 0.0],
        face: 1,
    },
    FaceInfo {
        position: [0.0, -1.0, -1.0],
        color: [0.0, 1.0, 0.0],
        face: 5,
    },
];

pub struct TextureGroup {
    pub color: glium::texture::Texture2d,
    pub normal: glium::texture::Texture2d,
    pub position: glium::texture::Texture2d,
    pub depth: glium::framebuffer::DepthRenderBuffer,
}

impl TextureGroup {
    fn new(disp: &glium::Display, (width, height): (u32, u32)) -> anyhow::Result<Self> {
        Ok(Self {
            color: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            normal: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F16F16F16,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            position: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F16F16F16,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            depth: glium::framebuffer::DepthRenderBuffer::new(
                disp,
                glium::texture::DepthFormat::F32,
                width,
                height,
            )?,
        })
    }

    fn as_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::MultiOutputFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        let color_attachments = [
            ("color", &self.color),
            ("normal", &self.normal),
            ("position", &self.position),
        ];
        glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            display,
            color_attachments.iter().cloned(),
            &self.depth,
        )
    }
}

pub struct GBufferRendererProvider {
    dimensions: (u32, u32),
    buffer: TextureGroup,
}

impl<'provider> SurfaceProvider<'provider> for GBufferRendererProvider {
    type Surface = glium::framebuffer::MultiOutputFrameBuffer<'provider>;
    type Output = &'provider TextureGroup;
    type Target = (Self::Surface, Self::Output);

    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        let dimensions = display.get_framebuffer_dimensions();
        Ok(Self {
            dimensions,
            buffer: TextureGroup::new(display, dimensions)?,
        })
    }

    fn get(
        &'provider mut self,
        display: &'provider glium::Display,
    ) -> anyhow::Result<Self::Target> {
        let dimensions = display.get_framebuffer_dimensions();
        if self.dimensions != dimensions {
            self.buffer = TextureGroup::new(display, dimensions)?;
            self.dimensions = dimensions;
        }
        let surface = self.buffer.as_surface(display)?;
        Ok((surface, &self.buffer))
    }
}

pub struct GBufferRenderer {
    vertex: glium::VertexBuffer<FaceInfo>,
    program: glium::Program,
}

#[inline(always)]
fn gen_face(
    world: &world::World,
    mapped: &mut WriteMapping<[FaceInfo]>,
    i: &mut usize,
    pos: WorldPosition,
    blk: &world::SolidBlock,
    direction: world::Direction,
) {
    if let Some(target_pos) = direction.apply(world.dims(), pos) {
        if world.test(target_pos) {
            return;
        }
    }

    mapped.set(
        *i,
        FaceInfo {
            position: pos.into(),
            color: blk.into(),
            face: direction.into(),
        },
    );
    *i += 1;
}

impl GBufferRenderer {
    fn build_vertex(
        &mut self,
        display: &glium::Display,
        world: &world::World,
    ) -> anyhow::Result<()> {
        let faces = world.dims().max_faces();
        if self.vertex.len() < faces {
            self.vertex = glium::VertexBuffer::empty_dynamic(display, faces)?;
        }
        let mut mapped = self.vertex.map_write();
        let mut i = 0usize;
        for (pos, blk) in world.iter() {
            for direction in world::Direction::iter() {
                gen_face(world, &mut mapped, &mut i, pos, blk, direction);
            }
        }
        Ok(())
    }
}

impl<'pass> Pass<'pass, GBufferRendererProvider> for GBufferRenderer {
    type Input = &'pass world::World;

    fn with_provider<'a>(
        display: &'a glium::Display,
        provider: GBufferRendererProvider,
    ) -> anyhow::Result<PassGroup<Self, GBufferRendererProvider>> {
        Ok(PassGroup::new(
            Self {
                vertex: glium::VertexBuffer::new(display, &CUBES)?,
                program: shader_program!(display, "cube" with geometry)?,
            },
            provider,
        ))
    }

    fn process<'surface>(
        &'pass mut self,
        display: &'surface glium::Display,
        surface: &'surface mut <GBufferRendererProvider as SurfaceProvider>::Surface,
        input: Self::Input,
    ) -> anyhow::Result<()> {
        self.build_vertex(display, input)?;
        surface.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        let aspect_ratio = {
            let dim = surface.get_dimensions();
            dim.0 as f32 / dim.1 as f32
        };
        let uniforms = Projection::new(
            aspect_ratio,
            glam::vec3(8.0, 10.0, 8.0),
            glam::vec3(20.0, 0.0, 20.0),
        )
        .to_uniform();
        surface.draw(
            self.vertex.slice(..).unwrap(),
            &glium::index::NoIndices(glium::index::PrimitiveType::Points),
            &self.program,
            &uniforms,
            &glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: glium::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            },
        )?;
        Ok(())
    }
}
