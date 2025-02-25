use {
  self::{
    analyzer::Analyzer, app::App, arguments::Arguments, bindings::Bindings, chain::Chain,
    error::Error, field::Field, filter::Filter, format::Format, frame::Frame, image::Image,
    input::Input, into_usize::IntoUsize, options::Options, program::Program, renderer::Renderer,
    shared::Shared, stream::Stream, subcommand::Subcommand, tally::Tally, target::Target,
    tiling::Tiling, track::Track, uniforms::Uniforms,
  },
  clap::{Parser, ValueEnum},
  cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfigRange,
  },
  log::info,
  rustfft::{num_complex::Complex, FftPlanner},
  skrifa::MetadataProvider,
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufWriter},
    path::{Path, PathBuf},
    process,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
  },
  vello::{
    kurbo,
    peniko::{self, Font},
  },
  wgpu::{
    include_wgsl, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, FragmentState,
    ImageSubresourceRange, Instance, Limits, LoadOp, Maintain, MaintainResult, MapMode,
    MemoryHints, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp,
    Surface, SurfaceConfiguration, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexState, COPY_BYTES_PER_ROW_ALIGNMENT,
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
mod arguments;
mod bindings;
mod chain;
mod error;
mod field;
mod filter;
mod format;
mod frame;
mod image;
mod input;
mod into_usize;
mod options;
mod program;
mod renderer;
mod shared;
mod stream;
mod subcommand;
mod tally;
mod target;
mod tiling;
mod track;
mod uniforms;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

const CHANNELS: u32 = 4;
const FONT: &str = "Helvetica Neue";

type Result<T = (), E = Error> = std::result::Result<T, E>;

type Mat3f = nalgebra::Matrix3<f32>;
type Mat4f = nalgebra::Matrix4<f32>;
type Vec2f = nalgebra::Vector2<f32>;
type Vec2u = nalgebra::Vector2<u32>;
type Vec4f = nalgebra::Vector4<f32>;

fn default<T: Default>() -> T {
  T::default()
}

fn invert_color() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

fn load_font(name: &str) -> Result<Font> {
  use font_kit::handle::Handle;

  let font = font_kit::source::SystemSource::new()
    .select_by_postscript_name(name)
    .context(error::FontSelection { name })?;

  let (font_data, font_index) = match font {
    Handle::Memory { bytes, font_index } => (bytes, font_index),
    Handle::Path { path, font_index } => (
      Arc::new(std::fs::read(&path).context(error::FilesystemIo { path })?),
      font_index,
    ),
  };

  Ok(Font::new(peniko::Blob::new(font_data), font_index))
}

fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}

fn main() {
  env_logger::init();

  if let Err(err) = Arguments::parse().run() {
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
