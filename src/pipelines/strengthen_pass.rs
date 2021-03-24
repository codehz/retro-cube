use glium::implement_uniform_block;

use crate::postprocess_shader_program;

use super::postprocess::SimplePostProcessPipeline;

pub struct StrengthenPass;

#[derive(Debug, Clone, Copy)]
pub struct StrengthenBlock {
    near: f32,
    far: f32,
}

implement_uniform_block!(StrengthenBlock, near, far);

impl SimplePostProcessPipeline for StrengthenPass {
    type Block = StrengthenBlock;

    fn load_shader(
        display: &glium::Display,
    ) -> Result<glium::Program, glium::ProgramCreationError> {
        postprocess_shader_program!(display, "strengthen")
    }

    fn get_block() -> Self::Block {
        StrengthenBlock {
            near: -10.0,
            far: 38.0,
        }
    }
}
