use super::*;

#[derive(Clone)]
pub(crate) struct Tap(Arc<RwLock<Core>>);

struct Core {
  buffer: Vec<f32>,
  decoder: Decoder<BufReader<File>>,
}

impl Tap {
  pub(crate) fn new(path: &Path) -> Result<Self> {
    let file = File::open(path).context(error::FilesystemIo { path })?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).context(error::DecoderOpen { path })?;
    Ok(Self(Arc::new(RwLock::new(Core {
      buffer: Vec::new(),
      decoder: source,
    }))))
  }
}

impl Source for Tap {
  fn current_frame_len(&self) -> Option<usize> {
    self.0.read().unwrap().decoder.current_frame_len()
  }

  fn channels(&self) -> u16 {
    self.0.read().unwrap().decoder.channels()
  }

  fn sample_rate(&self) -> u32 {
    self.0.read().unwrap().decoder.sample_rate()
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    self.0.read().unwrap().decoder.total_duration()
  }
}

impl Stream for Tap {
  fn drain(&mut self, samples: &mut Vec<f32>) {
    samples.append(&mut self.0.write().unwrap().buffer);
  }

  fn sample_rate(&self) -> u32 {
    self.0.read().unwrap().decoder.sample_rate()
  }
}

impl Iterator for Tap {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    let mut write = self.0.write().unwrap();

    let sample = write.decoder.next()?.to_sample::<f32>();

    write.buffer.push(sample);

    Some(sample)
  }
}
