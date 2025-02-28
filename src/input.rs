use super::*;

pub(crate) struct Input {
  stream_config: StreamConfig,
  queue: Arc<Mutex<VecDeque<f32>>>,
  #[allow(unused)]
  stream: cpal::Stream,
}

impl Input {
  pub(crate) fn new(device: rodio::Device, stream_config: SupportedStreamConfig) -> Result<Self> {
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    let clone = queue.clone();

    let buffer_size = match stream_config.buffer_size() {
      SupportedBufferSize::Range { min, .. } => {
        log::info!("input audio buffer size: {min}");
        Some(*min)
      }
      SupportedBufferSize::Unknown => {
        log::info!("input audio buffer size: unknown");
        None
      }
    };

    let mut stream_config = stream_config.config();

    if let Some(buffer_size) = buffer_size {
      stream_config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
    }

    let stream = device
      .build_input_stream(
        &stream_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
          clone.lock().unwrap().extend(data);
        },
        move |err| {
          eprintln!("audio input error: {err}");
        },
        None,
      )
      .context(error::AudioBuildInputStream)?;

    stream.play().context(error::AudioPlayStream)?;

    Ok(Self {
      stream_config,
      queue,
      stream,
    })
  }
}

impl Stream for Input {
  fn done(&self) -> bool {
    false
  }

  fn drain(&mut self, samples: &mut Vec<f32>) {
    samples.extend(self.queue.lock().unwrap().drain(..));
  }

  fn sample_rate(&self) -> u32 {
    self.stream_config.sample_rate.0
  }
}
