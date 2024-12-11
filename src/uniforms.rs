use super::*;

#[derive(Default)]
pub(crate) struct Uniforms {
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) filters: u32,
  pub(crate) fit: bool,
  pub(crate) image_read: bool,
  pub(crate) index: u32,
  pub(crate) offset: Vec2f,
  pub(crate) position: Mat3f,
  pub(crate) repeat: bool,
  pub(crate) resolution: Vec2f,
  pub(crate) source_offset: Vec2f,
  pub(crate) source_read: bool,
  pub(crate) tiling: u32,
  pub(crate) wrap: bool,
}

impl Uniforms {
  pub(crate) fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = 0;
    self.color.write(dst, &mut i, &mut a);
    self.coordinates.write(dst, &mut i, &mut a);
    self.field.write(dst, &mut i, &mut a);
    self.filters.write(dst, &mut i, &mut a);
    self.fit.write(dst, &mut i, &mut a);
    self.image_read.write(dst, &mut i, &mut a);
    self.index.write(dst, &mut i, &mut a);
    self.offset.write(dst, &mut i, &mut a);
    self.position.write(dst, &mut i, &mut a);
    self.repeat.write(dst, &mut i, &mut a);
    self.resolution.write(dst, &mut i, &mut a);
    self.source_offset.write(dst, &mut i, &mut a);
    self.source_read.write(dst, &mut i, &mut a);
    self.tiling.write(dst, &mut i, &mut a);
    self.wrap.write(dst, &mut i, &mut a);
    pad(i, a)
  }
}
