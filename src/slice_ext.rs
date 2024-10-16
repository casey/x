pub(crate) trait SliceExt {
  fn write(&mut self, data: &[u8]) -> &mut Self;
}

impl SliceExt for [u8] {
  fn write(&mut self, data: &[u8]) -> &mut Self {
    self[..data.len()].copy_from_slice(data);
    &mut self[data.len()..]
  }
}
