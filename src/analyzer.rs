use super::*;

pub(crate) struct Analyzer {
  complex_frequencies: Vec<Complex<f32>>,
  frequencies: Vec<f32>,
  planner: FftPlanner<f32>,
  rms: f32,
  samples: Vec<f32>,
  scratch: Vec<Complex<f32>>,
}

impl Analyzer {
  pub(crate) fn frequencies(&self) -> &[f32] {
    &self.frequencies
  }

  pub(crate) fn new() -> Self {
    Self {
      complex_frequencies: Vec::new(),
      frequencies: Vec::new(),
      planner: FftPlanner::new(),
      rms: 0.0,
      samples: Vec::new(),
      scratch: Vec::new(),
    }
  }

  pub(crate) fn rms(&self) -> f32 {
    self.rms
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }

  pub(crate) fn update(&mut self, stream: &mut dyn Stream, alpha: Parameter) {
    if stream.done() {
      self.samples.clear();
    } else {
      let old = self.samples.len();
      stream.drain(&mut self.samples);
      self
        .samples
        .drain(..self.samples.len().saturating_sub(128).min(old));
    }

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
    let spacing = stream.sample_rate() as f32 / n as f32;
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
    let alpha = alpha.unipolar();
    self.rms = alpha
      * (self.frequencies.iter().map(|&f| f * f).sum::<f32>()
        / self.frequencies.len().max(1) as f32)
        .sqrt()
      + (1.0 - alpha) * self.rms;
  }
}
