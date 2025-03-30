use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Value(pub(crate) u7);

impl Value {
  pub(crate) fn unipolar(self) -> f32 {
    f32::from(u8::from(self.0)) / 127.0
  }
}
