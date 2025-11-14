pub(crate) trait Stream {
  fn channels(&self) -> u16;

  fn done(&self) -> bool;

  fn drain(&mut self, samples: &mut Vec<f32>);

  fn sample_rate(&self) -> u32;
}
