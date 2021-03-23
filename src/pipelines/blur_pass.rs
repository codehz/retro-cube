use glium::implement_uniform_block;

use crate::postprocess_shader_program;

use super::{
    postprocess::{PostProcessPipeline, PostProcessProvider, SimplePostProcessPipeline},
    PassGroup, SurfaceProvider,
};

#[derive(Debug, Clone, Copy)]
pub struct BlurBlock {
    direction: [f32; 2],
}

implement_uniform_block!(BlurBlock, direction);

pub struct BlurPass<const DIR: bool>;

impl<const DIR: bool> SimplePostProcessPipeline for BlurPass<DIR> {
    type Block = BlurBlock;

    fn load_shader(
        display: &glium::Display,
    ) -> Result<glium::Program, glium::ProgramCreationError> {
        postprocess_shader_program!(display, "blur")
    }

    fn get_block() -> Self::Block {
        BlurBlock {
            direction: if DIR { [1.0, 1.0] } else { [1.0, -1.0] },
        }
    }
}

#[allow(dead_code)]
pub fn create_blur_pass<'pass, Provider: SurfaceProvider<'pass>>(
    display: &glium::Display,
) -> anyhow::Result<(
    PassGroup<PostProcessPipeline<BlurPass<false>>, PostProcessProvider>,
    PassGroup<PostProcessPipeline<BlurPass<true>>, Provider>,
)> {
    Ok((PassGroup::create(display)?, PassGroup::create(display)?))
}
