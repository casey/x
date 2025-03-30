use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Value(pub(crate) u7);

impl Value {
  pub(crate) fn parameter(self, min: f32, max: f32) -> f32 {
    min + (max - min) * f32::from(u8::from(self.0)) / 127.0
  }
}
