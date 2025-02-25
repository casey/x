use super::*;

#[derive(Tabled)]
struct StreamConfig {
  buffer_size: String,
  channels: u16,
  sample_format: SampleFormat,
  sample_rate: String,
}

impl From<SupportedStreamConfigRange> for StreamConfig {
  fn from(config: SupportedStreamConfigRange) -> Self {
    let min_sample_rate = config.min_sample_rate().0;
    let max_sample_rate = config.max_sample_rate().0;

    Self {
      sample_format: config.sample_format(),
      channels: config.channels(),
      sample_rate: if min_sample_rate == max_sample_rate {
        min_sample_rate.to_string()
      } else {
        format!("{min_sample_rate}–{max_sample_rate}")
      },
      buffer_size: match config.buffer_size() {
        SupportedBufferSize::Unknown => "unknown".into(),
        SupportedBufferSize::Range { min, max } => format!("{min}–{max}"),
      },
    }
  }
}

pub(crate) fn run() -> Result {
  let device = cpal::default_host()
    .default_output_device()
    .context(error::DefaultAudioOutputDevice)?;

  let output_configs = device
    .supported_output_configs()
    .context(error::SupportedStreamConfigs)?;

  println!(
    "{}",
    Table::new(output_configs.map(|config| StreamConfig::from(config)))
      .with(tabled::settings::style::Style::sharp())
  );

  Ok(())
}
