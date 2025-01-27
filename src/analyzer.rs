use super::*;

pub(crate) struct Analyzer {
  complex_frequencies: Vec<Complex<f32>>,
  config: StreamConfig,
  frequencies: Vec<f32>,
  planner: FftPlanner<f32>,
  queue: Arc<Mutex<VecDeque<f32>>>,
  samples: Vec<f32>,
  scratch: Vec<Complex<f32>>,
  #[allow(unused)]
  stream: cpal::Stream,
}

impl Analyzer {
  pub(crate) fn new() -> Result<Self> {
    let device = cpal::default_host()
      .default_input_device()
      .context(error::DefaultAudioInputDevice)?;

    let supported_config = device
      .supported_input_configs()
      .context(error::SupportedStreamConfigs)?
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::SupportedStreamConfig)?
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
      .context(error::BuildAudioInputStream)?;

    stream.play().context(error::PlayStream)?;

    Ok(Self {
      complex_frequencies: Vec::new(),
      config,
      frequencies: Vec::new(),
      planner: FftPlanner::new(),
      queue,
      samples: Vec::new(),
      scratch: Vec::new(),
      stream,
    })
  }

  pub(crate) fn frequencies(&self) -> &[f32] {
    &self.frequencies
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }

  pub(crate) fn update(&mut self) {
    self.samples.clear();
    self.samples.extend(self.queue.lock().unwrap().drain(..));

    let samples = &self.samples[..self.samples.len() & !1];

    self.complex_frequencies.clear();
    self
      .complex_frequencies
      .extend(samples.iter().map(Complex::from));
    let fft = self.planner.plan_fft_forward(samples.len());
    let scratch_len = fft.get_inplace_scratch_len();
    if self.scratch.len() < scratch_len {
      self.scratch.resize(scratch_len, 0.0.into());
    }
    fft.process_with_scratch(
      &mut self.complex_frequencies,
      &mut self.scratch[..scratch_len],
    );

    let n = self.complex_frequencies.len();
    let half = n / 2;
    let spacing = self.config.sample_rate.0 as f32 / n as f32;
    let threshold = (20.0 / spacing).into_usize();
    let cutoff = (15_000.0 / spacing).into_usize();

    self.frequencies.clear();
    self.frequencies.extend(
      self
        .complex_frequencies
        .iter()
        .enumerate()
        .skip(threshold)
        .take(cutoff.min(half).saturating_sub(threshold))
        .map(|(i, c)| {
          let weight = if i == 0 || i == half { 1.0 } else { 2.0 };
          c.norm() * weight
        }),
    );
  }
}
