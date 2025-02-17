pub(crate) trait Stream {
  fn drain(&mut self, samples: &mut Vec<f32>);

  fn sample_rate(&self) -> u32;
}
