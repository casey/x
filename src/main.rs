use {
  self::{
    analyzer::Analyzer, app::App, arguments::Arguments, bindings::Bindings, device::Device,
    error::Error, event::Event, field::Field, filter::Filter, format::Format, frame::Frame,
    hub::Hub, image::Image, input::Input, into_usize::IntoUsize, message::Message,
    options::Options, parameter::Parameter, program::Program, renderer::Renderer, shared::Shared,
    state::State, stream::Stream, subcommand::Subcommand, tally::Tally, target::Target,
    templates::ShaderWgsl, text::Text, tiling::Tiling, track::Track, uniforms::Uniforms,
  },
  boilerplate::Boilerplate,
  clap::{Parser, ValueEnum},
  log::info,
  regex::{Regex, RegexBuilder},
  rodio::{
    Decoder, OutputStream, Sink, Source,
    cpal::{
      self, Sample, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
      SupportedStreamConfigRange,
      traits::{DeviceTrait, HostTrait, StreamTrait},
    },
  },
  rustfft::{FftPlanner, num_complex::Complex},
  skrifa::MetadataProvider,
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufReader, BufWriter},
    num,
    ops::{Add, AddAssign, SubAssign},
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, mpsc},
    time::Instant,
  },
  strum::{EnumIter, IntoEnumIterator, IntoStaticStr},
  vello::{
    kurbo,
    peniko::{self, Font},
  },
  walkdir::WalkDir,
  wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoder, CommandEncoderDescriptor, DeviceDescriptor, Extent3d, Features, FragmentState,
    ImageSubresourceRange, Instance, Limits, LoadOp, Maintain, MaintainResult, MapMode,
    MemoryHints, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexState,
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
mod device;
mod error;
mod event;
mod field;
mod filter;
mod format;
mod frame;
mod hub;
mod image;
mod input;
mod into_usize;
mod message;
mod options;
mod parameter;
mod program;
mod renderer;
mod shared;
mod state;
mod stream;
mod subcommand;
mod tally;
mod target;
mod templates;
mod text;
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
