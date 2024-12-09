use {
  self::{
    app::App, bindings::Bindings, field::Field, filter::Filter, frame::Frame, options::Options,
    renderer::Renderer, shared::Shared, tally::Tally, target::Target, tiling::Tiling,
    uniforms::Uniforms,
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
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    Device, DeviceDescriptor, Extent3d, Features, FragmentState, ImageSubresourceRange, Instance,
    Limits, LoadOp, MemoryHints, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp,
    Surface, SurfaceConfiguration, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexState,
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
mod bindings;
mod field;
mod filter;
mod frame;
mod options;
mod renderer;
mod shared;
mod tally;
mod target;
mod tiling;
mod uniforms;

type Result<T = ()> = anyhow::Result<T>;

type Mat3f = nalgebra::Matrix3<f32>;
type Mat4f = nalgebra::Matrix4<f32>;
type Vec2f = nalgebra::Vector2<f32>;
type Vec2u = nalgebra::Vector2<u32>;
type Vec4f = nalgebra::Vector4<f32>;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

fn default<T: Default>() -> T {
  T::default()
}

fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
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
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
