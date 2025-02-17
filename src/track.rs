use {
  super::*,
  symphonia::core::{audio::Signal, io::MediaSourceStream, probe::Hint},
};

pub(crate) struct Track {
  index: usize,
  sample_rate: u32,
  samples: Vec<f32>,
  start: Instant,
}

impl Track {
  pub(crate) fn load(path: &Path) -> Result<Self> {
    let mut hint = Hint::new();

    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
      hint.with_extension(extension);
    }

    let file = File::open(path).context(error::FilesystemIo { path })?;

    let mss = MediaSourceStream::new(Box::new(file), default());

    let symphonia::core::probe::ProbeResult {
      format: mut reader, ..
    } = symphonia::default::get_probe()
      .format(&hint, mss, &default(), &default())
      .context(error::Track { path })?;

    let track = &reader.tracks()[0];

    let mut decoder = symphonia::default::get_codecs()
      .make(&track.codec_params, &default())
      .context(error::Track { path })?;

    let mut samples = Vec::<f32>::new();
    let mut sample_rate = None;

    loop {
      let packet = match reader.next_packet() {
        Err(symphonia::core::errors::Error::IoError(err))
          if err.kind() == io::ErrorKind::UnexpectedEof =>
        {
          break
        }
        result => result.context(error::Track { path })?,
      };

      let buffer = {
        let input = decoder.decode(&packet).context(error::Track { path })?;
        let mut output = input.make_equivalent::<f32>();
        input.convert(&mut output);
        output
      };

      let spec = buffer.spec();

      match sample_rate {
        Some(sample_rate) => {
          if spec.rate != sample_rate {
            return Err(error::SampleRateMismatch { path }.build());
          }
        }
        None => sample_rate = Some(spec.rate),
      }

      let base = samples.len();
      let channels = spec.channels.count();
      let frames = buffer.frames();

      samples.resize(base + frames, 0.0);

      for channel in 0..channels {
        let channel = buffer.chan(channel);
        for sample in 0..buffer.frames() {
          samples[base + sample] += channel[sample] / channels as f32;
        }
      }
    }

    Ok(Self {
      index: 0,
      sample_rate: sample_rate.context(error::EmptyTrack { path })?,
      samples,
      start: Instant::now(),
    })
  }

  fn index(&self, duration: Duration) -> usize {
    ((duration.as_secs_f64() * self.sample_rate as f64) as usize).min(self.samples.len())
  }
}

impl Stream for Track {
  fn drain(&mut self, samples: &mut Vec<f32>) {
    let end = Instant::now();

    let elapsed = end - self.start;

    let index = self.index(elapsed);

    eprintln!("{index}");

    samples.extend(&self.samples[self.index..index]);

    self.index = index;
  }

  fn sample_rate(&self) -> u32 {
    self.sample_rate
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
      if !($x - $y < $d || $y - $x < $d) {
        panic!();
      }
    };
  }

  // todo:
  // - test that stereo is summed and divided correctly

  #[test]
  fn zero() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();

    let path = temp.path().join("sine.wav");

    use hound;
    use std::f32::consts::PI;
    use std::i16;

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 16,
      sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for i in 0..100 {
      writer.write_sample(0);
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 100);

    for (i, actual) in track.samples.into_iter().enumerate() {
      assert_eq!(actual, 0.0);
    }
  }

  #[test]
  fn one() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();

    let path = temp.path().join("sine.wav");

    use hound;
    use std::f32::consts::PI;
    use std::i16;

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 16,
      sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for i in 0..100 {
      writer.write_sample(i16::MAX);
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 100);

    for (i, actual) in track.samples.into_iter().enumerate() {
      assert_eq!(actual, 0.9999695);
    }
  }

  #[test]
  fn load() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();

    let path = temp.path().join("sine.wav");

    use hound;
    use std::f32::consts::PI;
    use std::i16;

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 16,
      sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for t in (0..44100).map(|x| x as f32 / 44100.0) {
      let sample = (t * 440.0 * 2.0 * PI).sin();
      let amplitude = i16::MAX as f32;
      writer.write_sample((sample * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 44100);

    for (i, actual) in track.samples.into_iter().enumerate() {
      let expected = (i as f32 / 44100.0 * 440.0 * 2.0 * PI).sin();
      assert_delta!(actual, expected, 0.0001);
    }
  }

  #[test]
  fn downmix() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();

    let path = temp.path().join("sine.wav");

    use hound;
    use std::f32::consts::PI;
    use std::i16;

    let spec = hound::WavSpec {
      channels: 2,
      sample_rate: 44100,
      bits_per_sample: 16,
      sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for t in (0..44100).map(|x| x as f32 / 44100.0) {
      let value = (t * 440.0 * 2.0 * PI).sin();
      let amplitude = i16::MAX as f32;
      let sample = (value * amplitude) as i16;
      writer.write_sample(sample).unwrap();
      writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 44100);

    for (i, actual) in track.samples.into_iter().enumerate() {
      let expected = (i as f32 / 44100.0 * 440.0 * 2.0 * PI).sin();
      assert_delta!(actual, expected, 0.0001);
    }
  }
}
