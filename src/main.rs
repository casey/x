use {
  self::{
    analyzer::Analyzer, app::App, bindings::Bindings, error::Error, field::Field, filter::Filter,
    frame::Frame, into_usize::IntoUsize, options::Options, renderer::Renderer, shared::Shared,
    tally::Tally, target::Target, tiling::Tiling, uniforms::Uniforms,
  },
  clap::Parser,
  cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig, SupportedBufferSize, SupportedStreamConfigRange,
  },
  log::info,
  rustfft::{num_complex::Complex, FftPlanner},
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    process,
    sync::{mpsc, Arc, Mutex},
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

mod analyzer;
mod app;
mod bindings;
mod error;
mod field;
mod filter;
mod frame;
mod into_usize;
mod options;
mod renderer;
mod shared;
mod tally;
mod target;
mod tiling;
mod uniforms;

type Result<T = (), E = Error> = std::result::Result<T, E>;

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

fn invert_color() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}

fn run() -> Result<(), Error> {
  env_logger::init();

  let options = Options::parse();

  let mut app = App::new(options)?;

  EventLoop::with_user_event()
    .build()
    .context(error::EventLoopBuild)?
    .run_app(&mut app)
    .context(error::RunApp)?;

  if let Some(err) = app.error() {
    return Err(err);
  }

  Ok(())
}

fn main() {
  if let Err(err) = run() {
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = err.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }

    process::exit(1);
  }
}
