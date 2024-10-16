use super::*;

pub(crate) trait Shared {
  type Value;

  fn value(&self) -> Self::Value;
}

impl Shared for Field {
  type Value = [u8; 4];

  fn value(&self) -> Self::Value {
    (*self as u32).to_le_bytes()
  }
}

impl Shared for f32 {
  type Value = [u8; 4];

  fn value(&self) -> Self::Value {
    self.to_le_bytes()
  }
}
