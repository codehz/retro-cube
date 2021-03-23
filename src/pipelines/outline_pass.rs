use std::borrow::BorrowMut;

use glium::{uniform, Surface};

use crate::postprocess_shader_program;

use super::{
    gbuffer_pass::TextureGroup as GBufferTextureGroup, postprocess::*, Pass, PassGroup,
    SurfaceProvider,
};

pub struct OutlinePass {
    vertex: glium::VertexBuffer<PostProcessVertex>,
    program: glium::Program,
}

impl<'pass, Provider> Pass<'pass, Provider> for OutlinePass
where
    Provider: SurfaceProvider<'pass>,
{
    type Input = &'pass GBufferTextureGroup;

    fn with_provider<'display>(
        display: &'display glium::Display,
        provider: Provider,
    ) -> anyhow::Result<PassGroup<Self, Provider>> {
        Ok(PassGroup::new(
            Self {
                vertex: PostProcessVertex::get_buffer(display)?,
                program: postprocess_shader_program!(display, "outline")?,
            },
            provider,
        ))
    }

    fn process<'surface>(
        &'pass mut self,
        _display: &'surface glium::Display,
        surface: &'surface mut <Provider as SurfaceProvider<'pass>>::Surface,
        input: Self::Input,
    ) -> anyhow::Result<()> {
        let GBufferTextureGroup {
            color,
            normal,
            position,
            ..
        } = input;
        let color_sample = color
            .sampled()
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);
        let normal_sample = normal
            .sampled()
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);
        let position_sample = position
            .sampled()
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);
        let uniforms = uniform! {
            color_sample: color_sample,
            normal_sample: normal_sample,
            position_sample: position_sample,
        };
        surface.borrow_mut().draw(
            self.vertex.slice(..).unwrap(),
            glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
            &self.program,
            &uniforms,
            &glium::DrawParameters {
                smooth: Some(glium::Smooth::Nicest),
                ..Default::default()
            },
        )?;
        Ok(())
    }
}
