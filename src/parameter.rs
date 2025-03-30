use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parameter(pub(crate) u7);

impl Parameter {
  pub(crate) fn unipolar(self) -> f32 {
    f32::from(u8::from(self.0)) / 127.0
  }
}
