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

  fn play(self) -> Result {
    use rubato::{
      Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
    };
    let params = SincInterpolationParameters {
      sinc_len: 256,
      f_cutoff: 0.95,
      interpolation: SincInterpolationType::Linear,
      oversampling_factor: 256,
      window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler =
      SincFixedIn::<f64>::new(48000 as f64 / 44100 as f64, 2.0, params, 1024, 2).unwrap();

    let waves_in = vec![vec![0.0f64; 1024]; 2];
    let waves_out = resampler.process(&waves_in, None).unwrap();

    // todo:
    // - output format preferences:
    //   f32
    //   stereo
    //   high sample rate
    //   low buffer size
    // - do i really resample the whole track?
    //   - either i resample ahead of time
    //   - resample in an async thread which runs ahead
    //   - or resample as i go
    //   - really need to think about whether or not this is a good idea

    let device = cpal::default_host()
      .default_output_device()
      .context(error::DefaultAudioOutputDevice)?;

    let supported_config = device
      .supported_output_configs()
      .context(error::SupportedStreamConfigs)?
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::SupportedStreamConfig)?
      .with_max_sample_rate();

    let buffer_size = match supported_config.buffer_size() {
      SupportedBufferSize::Range { min, .. } => {
        log::info!("output audio buffer size: {min}");
        Some(*min)
      }
      SupportedBufferSize::Unknown => {
        log::info!("output audio buffer size: unknown");
        None
      }
    };

    let mut config = supported_config.config();

    if let Some(buffer_size) = buffer_size {
      config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
    }

    let mut start = 0;

    let stream = device
      .build_output_stream(
        &config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
          let end = (start + buffer.len()).min(self.samples.len());
          let samples = end - start;
          buffer[..samples].copy_from_slice(&self.samples[start..end]);
        },
        move |err| {
          eprintln!("audio output error: {err}");
        },
        None,
      )
      .context(error::BuildAudioStream)?;

    stream.play().context(error::PlayStream)?;

    loop {}

    Ok(())
  }

  fn index(&self, duration: Duration) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    ((duration.as_secs_f64() * self.sample_rate as f64) as usize).min(self.samples.len())
  }
}

impl Stream for Track {
  fn drain(&mut self, samples: &mut Vec<f32>) {
    let now = Instant::now();

    let elapsed = now - self.start;

    let index = self.index(elapsed);

    samples.extend(&self.samples[self.index..index]);

    self.index = index;
  }

  fn sample_rate(&self) -> u32 {
    self.sample_rate
  }
}

#[cfg(test)]
mod tests {
  use {super::*, hound::WavSpec, std::f32::consts::TAU, tempfile::TempDir};

  #[test]
  fn zero() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("foo.wav");

    let spec = WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 32,
      sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for _ in 0..100 {
      writer.write_sample(0.0).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 100);

    for sample in track.samples {
      assert_eq!(sample, 0.0);
    }
  }

  #[test]
  fn one() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("foo.wav");

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 32,
      sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for _ in 0..100 {
      writer.write_sample(1.0).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 100);

    for sample in track.samples {
      assert_eq!(sample, 1.0);
    }
  }

  #[test]
  fn sine() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("foo.wav");

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 32,
      sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for i in 0..44100 {
      writer
        .write_sample((i as f32 / 44100.0 * 440.0 * TAU).sin())
        .unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 44100);

    for (i, actual) in track.samples.into_iter().enumerate() {
      let expected = (i as f32 / 44100.0 * 440.0 * TAU).sin();
      assert_eq!(actual, expected);
    }
  }

  #[test]
  fn downmix() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("sine.wav");

    let spec = hound::WavSpec {
      channels: 2,
      sample_rate: 44100,
      bits_per_sample: 32,
      sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for _ in 0..100 {
      writer.write_sample(0.2).unwrap();
      writer.write_sample(0.8).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    assert_eq!(track.samples.len(), 100);

    for sample in track.samples {
      assert_eq!(sample, 0.5);
    }
  }

  #[test]
  fn play() {
    let temp = TempDir::new().unwrap();

    let path = temp.path().join("foo.wav");

    let spec = hound::WavSpec {
      channels: 1,
      sample_rate: 44100,
      bits_per_sample: 32,
      sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).unwrap();

    for i in 0..44100 {
      let t = i as f32 / 44100.0;
      writer.write_sample((t * 120.0 * TAU).sin()).unwrap();
    }

    writer.finalize().unwrap();

    let track = Track::load(&path).unwrap();

    track.play().unwrap();
  }
}
