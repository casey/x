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

#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
#[allow(clippy::arbitrary_source_item_ordering)]
struct MidiPort {
  number: usize,
  name: String,
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
  fn print_midi_port_table(input: bool, ports: Vec<MidiPort>) {
    println!(
      "{}",
      Table::new(ports)
        .with(Style::modern())
        .with(Panel::header(format!(
          "MIDI {}",
          if input { "input" } else { "output" }
        )))
        .with(BorderSpanCorrection)
    );
  }

  fn print_stream_table<T: Into<StreamConfig>, I: Iterator<Item = T>>(
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

  let midi_input = midir::MidiInput::new("MIDI Input").context(error::MidiInputInit)?;
  print_midi_port_table(
    true,
    midi_input
      .ports()
      .into_iter()
      .enumerate()
      .map(|(number, port)| {
        Ok(MidiPort {
          number,
          name: midi_input.port_name(&port)?,
        })
      })
      .collect::<Result<Vec<MidiPort>, midir::PortInfoError>>()
      .context(error::MidiPortInfo)?,
  );

  let midi_output = midir::MidiOutput::new("MIDI Output").context(error::MidiOutputInit)?;
  print_midi_port_table(
    true,
    midi_output
      .ports()
      .into_iter()
      .enumerate()
      .map(|(number, port)| {
        Ok(MidiPort {
          number,
          name: midi_output.port_name(&port)?,
        })
      })
      .collect::<Result<Vec<MidiPort>, midir::PortInfoError>>()
      .context(error::MidiPortInfo)?,
  );

  let host = cpal::default_host();

  for device in host.output_devices().context(error::AudioDevices)? {
    print_stream_table(
      &device.name().context(error::AudioDeviceName)?,
      false,
      device
        .supported_output_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    );
  }

  for device in host.input_devices().context(error::AudioDevices)? {
    print_stream_table(
      &device.name().context(error::AudioDeviceName)?,
      true,
      device
        .supported_input_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    );
  }

  Ok(())
}
