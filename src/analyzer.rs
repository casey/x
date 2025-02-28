use super::*;

pub(crate) struct Analyzer {
  complex_frequencies: Vec<Complex<f32>>,
  frequencies: Vec<f32>,
  planner: FftPlanner<f32>,
  samples: Vec<f32>,
  scratch: Vec<Complex<f32>>,
}

impl Analyzer {
  pub(crate) fn new() -> Self {
    Self {
      complex_frequencies: Vec::new(),
      frequencies: Vec::new(),
      planner: FftPlanner::new(),
      samples: Vec::new(),
      scratch: Vec::new(),
    }
  }

  pub(crate) fn frequencies(&self) -> &[f32] {
    &self.frequencies
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }

  pub(crate) fn update(&mut self, input: &mut dyn Stream) {
    if input.done() {
      self.samples.clear();
    } else {
      let old = self.samples.len();
      input.drain(&mut self.samples);
      self
        .samples
        .drain(..self.samples.len().saturating_sub(1024).min(old));
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
    let spacing = input.sample_rate() as f32 / n as f32;
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
