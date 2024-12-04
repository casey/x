use super::*;

pub(crate) struct Uniforms {
  pub(crate) field: Field,
  pub(crate) fit: bool,
  pub(crate) repeat: bool,
  pub(crate) resolution: Vec2f,
}

impl Uniforms {
  pub(crate) fn write(&self, dst: &mut [u8]) {
    let mut i = 0;
    self.field.write(dst, &mut i);
    self.fit.write(dst, &mut i);
    self.repeat.write(dst, &mut i);
    self.resolution.write(dst, &mut i);
  }

  pub(crate) fn buffer_size() -> u32 {
    let mut i = 0;
    Field::slot(&mut i);
    bool::slot(&mut i);
    bool::slot(&mut i);
    Vec2f::slot(&mut i);
    i.try_into().unwrap()
  }
}
