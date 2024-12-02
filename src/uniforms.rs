use super::*;

pub(crate) struct Uniforms {
  pub(crate) field: Field,
  pub(crate) resolution: f32,
}

impl Uniforms {
  pub(crate) const BUFFER_SIZE: u32 = (Field::SIZE + f32::SIZE) as u32;
}
