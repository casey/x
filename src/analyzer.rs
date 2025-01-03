use super::*;

const ALPHA: f32 = 0.9;

struct Input {
  config: StreamConfig,
  queue: Arc<Mutex<VecDeque<f32>>>,
  stream: cpal::Stream,
}

impl Input {
  fn new() -> Result<Self> {
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
          eprintln!("audio error: {err}");
        },
        None,
      )
      .context(error::BuildAudioInputStream)?;

    Ok(Self {
      config,
      queue,
      stream,
    })
  }

  fn play(&self) -> Result<()> {
    self.stream.play().context(error::PlayStream)
  }

  fn drain(&self, samples: &mut Vec<f32>) {
    samples.clear();
    samples.extend(self.queue.lock().unwrap().drain(..));
  }

  fn sample_rate(&self) -> u32 {
    self.config.sample_rate.0
  }
}

pub(crate) struct Analyzer {
  complex_frequencies: Vec<Complex<f32>>,
  dba: f32,
  frequencies: Vec<f32>,
  input: Input,
  planner: FftPlanner<f32>,
  samples: Vec<f32>,
  scratch: Vec<Complex<f32>>,
}

impl Analyzer {
  pub(crate) fn new() -> Result<Self> {
    let input = Input::new()?;

    input.play()?;

    Ok(Self {
      complex_frequencies: Vec::new(),
      dba: 0.0,
      frequencies: Vec::new(),
      input,
      planner: FftPlanner::new(),
      samples: Vec::new(),
      scratch: Vec::new(),
    })
  }

  pub(crate) fn frequencies(&self) -> &[f32] {
    &self.frequencies
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }

  pub(crate) fn update(&mut self) {
    self.input.drain(&mut self.samples);

    self.complex_frequencies.clear();
    self
      .complex_frequencies
      .extend(self.samples.iter().map(Complex::from));
    let fft = self.planner.plan_fft_forward(self.samples.len());
    let scratch_len = fft.get_inplace_scratch_len();
    if self.scratch.len() < scratch_len {
      self.scratch.resize(scratch_len, 0.0.into());
    }
    fft.process_with_scratch(
      &mut self.complex_frequencies,
      &mut self.scratch[..scratch_len],
    );

    self.frequencies.clear();
    self.frequencies.extend(
      self
        .complex_frequencies
        .iter()
        .map(|complex| complex.norm()),
    );

    let mut power = 0.0;
    let n = self.complex_frequencies.len() / 2;
    for (i, f) in self.complex_frequencies[..n].iter().enumerate() {
      let frequency =
        i as f32 * self.input.sample_rate() as f32 / self.complex_frequencies.len() as f32;
      power += f.norm_sqr() * fundsp::math::a_weight(frequency);
    }

    let dba = 10.0 * power.log10();

    if dba.classify() != FpCategory::Infinite {
      self.dba = ALPHA * dba + (1.0 - ALPHA) * self.dba;
    }
  }

  pub(crate) fn dba(&self) -> f32 {
    self.dba
  }
}
