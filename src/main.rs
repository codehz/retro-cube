use glium::glutin;
use pipelines::{ChainablePass, ProcessPass};

mod pipelines;
mod utils;
mod world;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let world = world::World::from_vox(&include_bytes!("../assets/test.vox")[..]);

    let mut pipeline = pipelines::PassGroup::<
        pipelines::gbuffer_pass::GBufferRenderer,
        pipelines::gbuffer_pass::GBufferRendererProvider,
    >::create(&display)
    .unwrap()
    .chain(
        pipelines::PassGroup::<
            pipelines::outline_pass::OutlinePass,
            pipelines::postprocess::PostProcessProvider,
        >::create(&display)
        .unwrap(),
    )
    .chain(
        pipelines::PassGroup::<
            pipelines::postprocess::PostProcessPipeline<pipelines::strengthen_pass::StrengthenPass>,
            pipelines::DisplaySurfaceProvider,
        >::create(&display)
        .unwrap(),
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                glutin::event::StartCause::Poll => (),
                _ => return,
            },
            _ => return,
        }

        pipeline
            .process(&display, &world)
            .unwrap()
            .swapchains()
            .unwrap();
    });
}
