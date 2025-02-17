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

      dbg!(samples.len());

      let buffer = decoder.decode(&packet).context(error::Track { path })?;
      let buffer = buffer.make_equivalent::<f32>();

      match sample_rate {
        Some(sample_rate) => {
          if buffer.spec().rate != sample_rate {
            return Err(error::SampleRateMismatch { path }.build());
          }
        }
        None => sample_rate = Some(buffer.spec().rate),
      }

      dbg!(buffer.chan(0).len());

      samples.extend(buffer.chan(0));
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
