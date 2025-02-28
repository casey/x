use super::*;

#[derive(Default)]
pub(crate) struct Uniforms {
  pub(crate) back_read: bool,
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) filters: u32,
  pub(crate) fit: bool,
  pub(crate) frequency_range: f32,
  pub(crate) front_offset: Vec2f,
  pub(crate) front_read: bool,
  pub(crate) gain: f32,
  pub(crate) index: u32,
  pub(crate) offset: Vec2f,
  pub(crate) position: Mat3f,
  pub(crate) repeat: bool,
  pub(crate) resolution: Vec2f,
  pub(crate) sample_range: f32,
  pub(crate) tiling: u32,
  pub(crate) wrap: bool,
}

impl Uniforms {
  pub(crate) fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = 0;
    self.back_read.write(dst, &mut i, &mut a);
    self.color.write(dst, &mut i, &mut a);
    self.coordinates.write(dst, &mut i, &mut a);
    self.field.write(dst, &mut i, &mut a);
    self.filters.write(dst, &mut i, &mut a);
    self.fit.write(dst, &mut i, &mut a);
    self.frequency_range.write(dst, &mut i, &mut a);
    self.front_offset.write(dst, &mut i, &mut a);
    self.front_read.write(dst, &mut i, &mut a);
    self.gain.write(dst, &mut i, &mut a);
    self.index.write(dst, &mut i, &mut a);
    self.offset.write(dst, &mut i, &mut a);
    self.position.write(dst, &mut i, &mut a);
    self.repeat.write(dst, &mut i, &mut a);
    self.resolution.write(dst, &mut i, &mut a);
    self.sample_range.write(dst, &mut i, &mut a);
    self.tiling.write(dst, &mut i, &mut a);
    self.wrap.write(dst, &mut i, &mut a);
    pad(i, a)
  }
}
