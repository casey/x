use super::*;

pub(crate) struct Input {
  config: StreamConfig,
  queue: Arc<Mutex<VecDeque<f32>>>,
  #[allow(unused)]
  stream: cpal::Stream,
}

impl Input {
  pub(crate) fn new() -> Result<Self> {
    let device = cpal::default_host()
      .default_input_device()
      .context(error::AudioDefaultInputDevice)?;

    let supported_config = device
      .supported_input_configs()
      .context(error::AudioSupportedStreamConfigs)?
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::AudioSupportedStreamConfig)?
      .with_max_sample_rate();

    let buffer_size = match supported_config.buffer_size() {
      SupportedBufferSize::Range { min, .. } => {
        log::info!("input audio buffer size: {min}");
        Some(*min)
      }
      SupportedBufferSize::Unknown => {
        log::info!("input audio buffer size: unknown");
        None
      }
    };

    let mut config = supported_config.config();

    if let Some(buffer_size) = buffer_size {
      config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
    }

    let queue = Arc::new(Mutex::new(VecDeque::new()));

    let clone = queue.clone();

    let stream = device
      .build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
          clone.lock().unwrap().extend(data);
        },
        move |err| {
          eprintln!("audio input error: {err}");
        },
        None,
      )
      .context(error::AudioBuildStream)?;

    stream.play().context(error::AudioPlayStream)?;

    Ok(Self {
      config,
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
    self.config.sample_rate.0
  }
}
