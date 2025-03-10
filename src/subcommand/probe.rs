use {
  super::*,
  tabled::{
    settings::{
      style::{BorderSpanCorrection, Style},
      Panel,
    },
    Table, Tabled,
  },
};

#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
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
  fn print_table<T: Into<StreamConfig>, I: Iterator<Item = T>>(
    name: &str,
    input: bool,
    configs: I,
  ) {
    println!(
      "{}",
      Table::new(configs.map(Into::into))
        .with(Style::modern())
        .with(Panel::header(format!(
          "{name} ({})",
          if input { "input" } else { "output" }
        )))
        .with(BorderSpanCorrection)
    );
  }

  let host = cpal::default_host();

  for device in host.output_devices().context(error::AudioDevices)? {
    print_table(
      &device.name().context(error::AudioDeviceName)?,
      false,
      device
        .supported_output_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    );
  }

  for device in host.input_devices().context(error::AudioDevices)? {
    print_table(
      &device.name().context(error::AudioDeviceName)?,
      true,
      device
        .supported_input_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    );
  }

  Ok(())
}
