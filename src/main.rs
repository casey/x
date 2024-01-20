use {
  self::renderer::Renderer,
  std::borrow::Cow,
  wgpu::{
    Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, Instance,
    Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PowerPreference,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp,
    Surface, SurfaceConfiguration, TextureViewDescriptor, VertexState,
  },
  winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
  },
};

mod renderer;

// todo:
// - error handling with backtrace and line number

fn main() {
  env_logger::init();

  let event_loop = EventLoop::new().unwrap();

  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let mut renderer = pollster::block_on(Renderer::new(&window));

  event_loop
    .run(|event, target| renderer.handle_event(event, target))
    .unwrap();
}
