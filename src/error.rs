use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to get adapter"))]
  Adapter { backtrace: Option<Backtrace> },
  #[snafu(display("failed to build audio input stream"))]
  BuildAudioInputStream {
    backtrace: Option<Backtrace>,
    source: cpal::BuildStreamError,
  },
  #[snafu(display("failed to map capture buffer"))]
  CaptureBufferMap {
    backtrace: Option<Backtrace>,
    source: wgpu::BufferAsyncError,
  },
  #[snafu(display("failed to create overlay renderer"))]
  CreateOverlayRenderer {
    backtrace: Option<Backtrace>,
    source: vello::Error,
  },
  #[snafu(display("failed to create surface"))]
  CreateSurface {
    backtrace: Option<Backtrace>,
    source: wgpu::CreateSurfaceError,
  },
  #[snafu(display("failed to create window"))]
  CreateWindow {
    backtrace: Option<Backtrace>,
    source: winit::error::OsError,
  },
  #[snafu(display("failed to get current texture"))]
  CurrentTexture {
    backtrace: Option<Backtrace>,
    source: wgpu::SurfaceError,
  },
  #[snafu(display("failed to get default audio input device"))]
  DefaultAudioInputDevice { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get default config"))]
  DefaultConfig { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get device"))]
  Device {
    backtrace: Option<Backtrace>,
    source: wgpu::RequestDeviceError,
  },
  #[snafu(display("empty track: {}", path.display()))]
  EmptyTrack {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("failed to build event loop"))]
  EventLoopBuild {
    backtrace: Option<Backtrace>,
    source: winit::error::EventLoopError,
  },
  #[snafu(display("could not retrieve glyph for character `{character}`"))]
  FontGlyph {
    backtrace: Option<Backtrace>,
    character: char,
  },
  #[snafu(display("failed to read font"))]
  FontRead {
    backtrace: Option<Backtrace>,
    source: skrifa::outline::error::ReadError,
  },
  #[snafu(display("failed to select font `{name}`"))]
  FontSelection {
    backtrace: Option<Backtrace>,
    name: String,
    source: font_kit::error::SelectionError,
  },
  #[snafu(display("I/O error at `{}`", path.display()))]
  FilesystemIo {
    path: PathBuf,
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("internal error: {message}"))]
  Internal {
    backtrace: Option<Backtrace>,
    message: String,
  },
  #[snafu(display("failed to play audio input stream"))]
  PlayStream {
    backtrace: Option<Backtrace>,
    source: cpal::PlayStreamError,
  },
  #[snafu(display("failed to encode png at {}", path.display()))]
  PngEncode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
    source: png::EncodingError,
  },
  #[snafu(display("failed to render overlay"))]
  RenderOverlay {
    backtrace: Option<Backtrace>,
    source: vello::Error,
  },
  #[snafu(display("failed to run app"))]
  RunApp {
    backtrace: Option<Backtrace>,
    source: winit::error::EventLoopError,
  },
  #[snafu(display("track packet sample rate mismatch: {}", path.display()))]
  SampleRateMismatch {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("failed to get supported stream config"))]
  SupportedStreamConfig { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get supported stream configs"))]
  SupportedStreamConfigs {
    backtrace: Option<Backtrace>,
    source: cpal::SupportedStreamConfigsError,
  },
  #[snafu(display("failed to decode track"))]
  Track {
    backtrace: Option<Backtrace>,
    path: PathBuf,
    source: symphonia::core::errors::Error,
  },
  #[snafu(display("default texture format {texture_format:?} not supported"))]
  UnsupportedTextureFormat {
    backtrace: Option<Backtrace>,
    texture_format: TextureFormat,
  },
  #[snafu(display("validation failed"))]
  Validation {
    backtrace: Option<Backtrace>,
    source: wgpu::Error,
  },
}

impl Error {
  pub(crate) fn internal(message: impl Into<String>) -> Self {
    Internal { message }.build()
  }
}
