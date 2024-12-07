use {
  self::{
    app::App, field::Field, filter::Filter, frame::Frame, options::Options, renderer::Renderer,
    shared::Shared, tally::Tally, target::Target, uniforms::Uniforms, vec2f::Vec2f,
  },
  anyhow::Context,
  clap::Parser,
  log::info,
  std::{
    backtrace::BacktraceStatus,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    process,
    sync::Arc,
    time::Instant,
  },
  wgpu::{
    include_wgsl, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, Color,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, FragmentState,
    Instance, Limits, LoadOp, MemoryHints, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp,
    Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexState,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
  },
};

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

mod app;
mod field;
mod filter;
mod frame;
mod options;
mod renderer;
mod shared;
mod tally;
mod target;
mod uniforms;
mod vec2f;

type Result<T = ()> = anyhow::Result<T>;

fn default<T: Default>() -> T {
  T::default()
}

fn run() -> Result<()> {
  env_logger::init();

  let options = Options::parse();

  let mut app = App::new(options);

  EventLoop::with_user_event().build()?.run_app(&mut app)?;

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
