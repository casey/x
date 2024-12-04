use {
  super::*,
  clap::builder::styling::{AnsiColor, Effects, Styles},
};

#[derive(Clone, Parser)]
#[command(
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
  pub(crate) fit: bool,
  #[arg(long)]
  pub(crate) repeat: bool,
  #[arg(long)]
  pub(crate) resolution: Option<u32>,
}

impl Options {
  pub(crate) fn resolution(&self, window_size: PhysicalSize<u32>) -> u32 {
    self
      .resolution
      .unwrap_or(window_size.height.max(window_size.width))
      .max(1)
  }
}