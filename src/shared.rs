use super::*;

pub(crate) trait Shared {
  const SIZE: usize;

  type Value;

  fn value(&self) -> Self::Value;
}

impl Shared for Field {
  const SIZE: usize = 4;

  type Value = [u8; Self::SIZE];

  fn value(&self) -> Self::Value {
    (*self as u32).to_le_bytes()
  }
}

impl Shared for f32 {
  const SIZE: usize = 4;

  type Value = [u8; Self::SIZE];

  fn value(&self) -> Self::Value {
    self.to_le_bytes()
  }
}
