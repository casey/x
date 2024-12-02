use {
  self::{
    app::App, field::Field, filter::Filter, frame::Frame, options::Options, renderer::Renderer,
    shared::Shared, slice_ext::SliceExt, tally::Tally, target::Target, uniforms::Uniforms,
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
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
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
    keyboard::Key,
    window::{Window, WindowAttributes, WindowId},
  },
};

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

type Result<T = ()> = anyhow::Result<T>;

mod app;
mod field;
mod filter;
mod frame;
mod options;
mod renderer;
mod shared;
mod slice_ext;
mod tally;
mod target;
mod uniforms;

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
