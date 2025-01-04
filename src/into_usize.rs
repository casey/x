pub(crate) trait IntoUsize {
  fn into_usize(self) -> usize;
}

impl IntoUsize for u32 {
  fn into_usize(self) -> usize {
    self.try_into().unwrap()
  }
}

impl IntoUsize for f32 {
  fn into_usize(self) -> usize {
    #![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    self as usize
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn u32_into_usize() {
    u32::MAX.into_usize();
  }
}
