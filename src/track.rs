use super::*;

#[derive(Clone)]
pub(crate) struct Track(Arc<RwLock<Inner>>);

struct Inner {
  buffer: Vec<f32>,
  decoder: Decoder<BufReader<File>>,
  done: bool,
}

impl Track {
  pub(crate) fn new(path: &Path) -> Result<Self> {
    let file = File::open(path).context(error::FilesystemIo { path })?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).context(error::DecoderOpen { path })?;
    Ok(Self(Arc::new(RwLock::new(Inner {
      buffer: Vec::new(),
      decoder: source,
      done: false,
    }))))
  }

  fn read(&self) -> RwLockReadGuard<Inner> {
    self.0.read().unwrap()
  }

  fn write(&mut self) -> RwLockWriteGuard<Inner> {
    self.0.write().unwrap()
  }
}

impl Source for Track {
  fn current_frame_len(&self) -> Option<usize> {
    self.read().decoder.current_frame_len()
  }

  fn channels(&self) -> u16 {
    self.read().decoder.channels()
  }

  fn sample_rate(&self) -> u32 {
    self.read().decoder.sample_rate()
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    self.read().decoder.total_duration()
  }
}

impl Stream for Track {
  fn done(&self) -> bool {
    let inner = self.read();
    inner.done && inner.buffer.is_empty()
  }

  fn drain(&mut self, samples: &mut Vec<f32>) {
    let mut inner = self.write();
    let channels = inner.decoder.channels();

    samples.extend(
      inner
        .buffer
        .chunks(inner.decoder.channels().into())
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32),
    );

    inner.buffer.clear();
  }

  fn sample_rate(&self) -> u32 {
    self.read().decoder.sample_rate()
  }
}

impl Iterator for Track {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    let mut inner = self.write();

    let Some(sample) = inner.decoder.next() else {
      inner.done = true;
      return None;
    };

    let sample = sample.to_sample();

    inner.buffer.push(sample);

    Some(sample)
  }
}
