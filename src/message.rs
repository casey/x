use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum MessageParseError {
  EventParse {
    source: midly::Error,
  },
  UnrecognizedEvent {
    event: midly::live::LiveEvent<'static>,
  },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Message {
  pub(crate) control: u8,
  pub(crate) device: Device,
  pub(crate) event: Event,
  pub(crate) timestamp: u64,
}

impl Message {
  pub(crate) fn parse(timestamp: u64, event: &[u8]) -> Result<Self, MessageParseError> {
    let event = midly::live::LiveEvent::parse(event).context(EventParseError)?;
    let (channel, key, value, press): (u8, u8, u8, bool) = match event {
      midly::live::LiveEvent::Midi { channel, message } => match message {
        midly::MidiMessage::NoteOn { key, vel } => (channel.into(), key.into(), vel.into(), true),
        midly::MidiMessage::NoteOff { key, vel } => (channel.into(), key.into(), vel.into(), false),
        _ => {
          return Err(MessageParseError::UnrecognizedEvent {
            event: event.to_static(),
          })
        }
      },
      _ => {
        return Err(MessageParseError::UnrecognizedEvent {
          event: event.to_static(),
        })
      }
    };

    let (device, control, event) = match (channel, key) {
      (0, 0..=15) => (Device::Twister, key, Event::Encoder { value }),
      (1, 0..=15) => (Device::Twister, key, Event::Button { press }),
      (2, 36..=51) => (
        Device::Spectra,
        match key {
          48 => 0,
          49 => 1,
          50 => 2,
          51 => 3,
          44 => 4,
          45 => 5,
          46 => 6,
          47 => 7,
          40 => 8,
          41 => 9,
          42 => 10,
          43 => 11,
          36 => 12,
          37 => 13,
          38 => 14,
          39 => 15,
          _ => unreachable!(),
        },
        Event::Button { press },
      ),
      (3, 20..=25) => (
        Device::Spectra,
        match key {
          22 => 16,
          21 => 17,
          20 => 18,
          25 => 19,
          24 => 20,
          23 => 21,
          _ => unreachable!(),
        },
        Event::Button { press },
      ),
      (3, 8..=13) => (Device::Twister, key - 8 + 16, Event::Button { press }),
      _ => {
        return Err(MessageParseError::UnrecognizedEvent {
          event: event.to_static(),
        })
      }
    };

    Ok(Self {
      control,
      device,
      event,
      timestamp,
    })
  }
}
