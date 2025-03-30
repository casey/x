use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to get adapter"))]
  Adapter { backtrace: Option<Backtrace> },
  #[snafu(display("failed to build audio input stream"))]
  AudioBuildInputStream {
    backtrace: Option<Backtrace>,
    source: cpal::BuildStreamError,
  },
  #[snafu(display("failed to get default audio output stream"))]
  AudioBuildOutputStream {
    backtrace: Option<Backtrace>,
    source: rodio::StreamError,
  },
  #[snafu(display("failed to get default audio input device"))]
  AudioDefaultInputDevice { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get default audio output device"))]
  AudioDefaultOutputDevice { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get audio device name"))]
  AudioDeviceName {
    backtrace: Option<Backtrace>,
    source: cpal::DeviceNameError,
  },
  #[snafu(display("failed to enumerate audio devices"))]
  AudioDevices {
    backtrace: Option<Backtrace>,
    source: cpal::DevicesError,
  },
  #[snafu(display("failed to play audio"))]
  AudioPlay {
    backtrace: Option<Backtrace>,
    source: rodio::PlayError,
  },
  #[snafu(display("failed to play audio input stream"))]
  AudioPlayStream {
    backtrace: Option<Backtrace>,
    source: cpal::PlayStreamError,
  },
  #[snafu(display("failed to get supported stream config"))]
  AudioSupportedStreamConfig { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get supported stream configs"))]
  AudioSupportedStreamConfigs {
    backtrace: Option<Backtrace>,
    source: cpal::SupportedStreamConfigsError,
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
  #[snafu(display("failed to open audio file"))]
  DecoderOpen {
    backtrace: Option<Backtrace>,
    path: PathBuf,
    source: rodio::decoder::DecoderError,
  },
  #[snafu(display("failed to get default config"))]
  DefaultConfig { backtrace: Option<Backtrace> },
  #[snafu(display("failed to get device"))]
  Device {
    backtrace: Option<Backtrace>,
    source: wgpu::RequestDeviceError,
  },
  #[snafu(display("failed to build event loop"))]
  EventLoopBuild {
    backtrace: Option<Backtrace>,
    source: winit::error::EventLoopError,
  },
  #[snafu(display("I/O error at `{}`", path.display()))]
  FilesystemIo {
    path: PathBuf,
    backtrace: Option<Backtrace>,
    source: io::Error,
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
  #[snafu(display("could not get home directory"))]
  Home { backtrace: Option<Backtrace> },
  #[snafu(display("internal error: {message}"))]
  Internal {
    backtrace: Option<Backtrace>,
    message: String,
  },
  #[snafu(display("failed to initialize MIDI input"))]
  MidiInputInit {
    backtrace: Option<Backtrace>,
    source: midir::InitError,
  },
  #[snafu(display("failed to connect to MIDI port"))]
  MidiInputPortConnect {
    backtrace: Option<Backtrace>,
    source: midir::ConnectError<midir::MidiInput>,
  },
  #[snafu(display("failed to initialize MIDI output"))]
  MidiOutputInit {
    backtrace: Option<Backtrace>,
    source: midir::InitError,
  },
  #[snafu(display("failed to get MIDI port info"))]
  MidiPortInfo {
    backtrace: Option<Backtrace>,
    source: midir::PortInfoError,
  },
  #[snafu(display("failed to decode png at {}", path.display()))]
  PngDecode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
    source: png::DecodingError,
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
  #[snafu(
    display(
      "more than one match for song: {}",
      matches.iter().map(|path| path.display().to_string()).collect::<Vec<String>>().join(", ")
    )
  )]
  SongAmbiguous {
    backtrace: Option<Backtrace>,
    matches: Vec<PathBuf>,
  },
  #[snafu(display("could not match song `{song}`"))]
  SongMatch {
    backtrace: Option<Backtrace>,
    song: Regex,
  },
  #[snafu(display("invalid song regex"))]
  SongRegex {
    backtrace: Option<Backtrace>,
    source: regex::Error,
  },
  #[snafu(display("I/O error finding song"))]
  SongWalk {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
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
