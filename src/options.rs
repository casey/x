use {
  super::*,
  clap::{
    builder::styling::{AnsiColor, Effects, Styles},
    ArgGroup,
  },
};

#[derive(Clone, Default, Parser)]
#[command(
  group(ArgGroup::new("audio").args(["input", "song", "track"])),
  version,
  styles = Styles::styled()
    .error(AnsiColor::Red.on_default() | Effects::BOLD)
    .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .invalid(AnsiColor::Red.on_default())
    .literal(AnsiColor::Blue.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .valid(AnsiColor::Green.on_default())
)]
pub(crate) struct Options {
  #[arg(long)]
  pub(crate) db: Option<f32>,
  #[arg(long)]
  pub(crate) fit: bool,
  #[arg(long)]
  pub(crate) input: bool,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) repeat: bool,
  #[arg(long)]
  pub(crate) resolution: Option<u32>,
  #[arg(long)]
  pub(crate) song: Option<String>,
  #[arg(long)]
  pub(crate) tile: bool,
  #[arg(long)]
  pub(crate) track: Option<PathBuf>,
  #[arg(long)]
  pub(crate) volume: Option<f32>,
}

impl Options {
  pub(crate) fn resolution(&self, window_size: PhysicalSize<u32>) -> u32 {
    self
      .resolution
      .unwrap_or(window_size.height.max(window_size.width))
      .max(1)
  }
}
