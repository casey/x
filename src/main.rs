use {
  self::{app::App, renderer::Renderer},
  anyhow::Context,
  std::{backtrace::BacktraceStatus, process, sync::Arc},
  wgpu::{
    include_wgsl, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    FragmentState, Instance, Limits, LoadOp, MemoryHints, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor,
    VertexState,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
  },
};

type Result<T = ()> = anyhow::Result<T>;

mod app;
mod renderer;

fn run() -> Result<()> {
  env_logger::init();

  let event_loop = EventLoop::new()?;

  let mut app = App::default();

  event_loop.run_app(&mut app)?;

  if let Some(err) = app.error() {
    return Err(err);
  }

  Ok(())
}

fn main() {
  if let Err(error) = run() {
    eprintln!("error: {error}");

    let backtrace = error.backtrace();

    if let BacktraceStatus::Captured = backtrace.status() {
      eprintln!("{}", backtrace);
    }

    process::exit(1);
  }
}
