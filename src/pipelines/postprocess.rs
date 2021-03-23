use glium::{implement_vertex, uniform, Surface};

use super::{Pass, PassGroup, SurfaceProvider};

#[derive(Copy, Clone)]
pub struct PostProcessVertex {
    id: u32,
}

implement_vertex!(PostProcessVertex, id);

impl PostProcessVertex {
    fn get() -> &'static [PostProcessVertex; 4] {
        &[
            PostProcessVertex { id: 0 },
            PostProcessVertex { id: 1 },
            PostProcessVertex { id: 2 },
            PostProcessVertex { id: 3 },
        ]
    }

    pub fn get_buffer(
        display: &glium::Display,
    ) -> Result<glium::VertexBuffer<PostProcessVertex>, glium::vertex::BufferCreationError> {
        glium::VertexBuffer::new(display, PostProcessVertex::get())
    }
}

struct TextureGroup(glium::texture::Texture2d);

impl TextureGroup {
    fn new(display: &glium::Display, (width, height): (u32, u32)) -> anyhow::Result<Self> {
        Ok(Self(glium::texture::Texture2d::empty_with_format(
            display,
            glium::texture::UncompressedFloatFormat::F16F16F16F16,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height,
        )?))
    }

    fn as_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::SimpleFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        glium::framebuffer::SimpleFrameBuffer::new(display, &self.0)
    }
}

pub struct PostProcessProvider {
    dimensions: (u32, u32),
    group: TextureGroup,
}

impl<'provider> SurfaceProvider<'provider> for PostProcessProvider {
    type Surface = glium::framebuffer::SimpleFrameBuffer<'provider>;
    type Output = &'provider glium::texture::Texture2d;
    type Target = (Self::Surface, Self::Output);

    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        let dimensions = display.get_framebuffer_dimensions();
        Ok(Self {
            dimensions,
            group: TextureGroup::new(display, dimensions)?,
        })
    }

    fn get(
        &'provider mut self,
        display: &'provider glium::Display,
    ) -> anyhow::Result<Self::Target> {
        let dimensions = display.get_framebuffer_dimensions();
        if self.dimensions != dimensions {
            self.group = TextureGroup::new(display, dimensions)?;
            self.dimensions = dimensions;
        }
        let surface = self.group.as_surface(display)?;
        Ok((surface, &self.group.0))
    }
}

pub trait SimplePostProcessPipeline {
    type Block: glium::uniforms::UniformBlock + glium::buffer::Content + Copy;

    fn load_shader(
        display: &glium::Display,
    ) -> Result<glium::Program, glium::program::ProgramCreationError>;

    fn get_block() -> Self::Block;
}

pub struct PostProcessPipeline<T: SimplePostProcessPipeline> {
    _impl: std::marker::PhantomData<T>,
    vertex: glium::VertexBuffer<PostProcessVertex>,
    program: glium::Program,
}

impl<'pass, T, Provider> Pass<'pass, Provider> for PostProcessPipeline<T>
where
    T: SimplePostProcessPipeline,
    Provider: SurfaceProvider<'pass>,
{
    type Input = &'pass glium::texture::Texture2d;

    fn with_provider<'display>(
        display: &'display glium::Display,
        provider: Provider,
    ) -> anyhow::Result<PassGroup<Self, Provider>> {
        Ok(PassGroup::new(
            Self {
                _impl: std::marker::PhantomData,
                vertex: PostProcessVertex::get_buffer(display)?,
                program: T::load_shader(display)?,
            },
            provider,
        ))
    }

    fn process<'surface>(
        &'pass mut self,
        display: &'surface glium::Display,
        surface: &'surface mut <Provider as SurfaceProvider<'pass>>::Surface,
        input: Self::Input,
    ) -> anyhow::Result<()> {
        let block = T::get_block();
        let block = glium::uniforms::UniformBuffer::new(display, block)?;
        let sample = input
            .sampled()
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::MirrorClamp);
        let uniforms = uniform! {
            color_sample: sample,
            block: &block,
        };

        surface.draw(
            self.vertex.slice(..).unwrap(),
            glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
            &self.program,
            &uniforms,
            &Default::default(),
        )?;
        Ok(())
    }
}
