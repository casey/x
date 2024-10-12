use {
  self::renderer::Renderer,
  anyhow::Context,
  std::{backtrace::BacktraceStatus, borrow::Cow, process},
  wgpu::{
    Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, Instance,
    Limits, LoadOp, MemoryHints, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
    TextureViewDescriptor, VertexState,
  },
  winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
  },
};

type Result<T = ()> = anyhow::Result<T>;

mod renderer;

fn handle_error(result: Result) {
  if let Err(error) = result {
    eprintln!("error: {error}");

    let backtrace = error.backtrace();

    if let BacktraceStatus::Captured = backtrace.status() {
      eprintln!("{}", backtrace);
    }

    process::exit(1);
  }
}

fn run() -> Result<()> {
  env_logger::init();

  let event_loop = EventLoop::new()?;

  let window = WindowBuilder::new().with_title("x").build(&event_loop)?;

  let mut renderer = pollster::block_on(Renderer::new(&window))?;

  event_loop.run(|event, target| handle_error(renderer.handle_event(event, target)))?;

  Ok(())
}

fn main() {
  handle_error(run());
}
