use super::*;

pub(crate) trait Shared {
  const ALIGNMENT: usize;
  const SIZE: usize;

  fn write(&self, buffer: &mut [u8], i: &mut usize) {
    let start = Self::slot(i);
    self.write_aligned(&mut buffer[start..*i]);
  }

  fn slot(i: &mut usize) -> usize {
    assert!(Self::ALIGNMENT.is_power_of_two());
    let start = (*i + Self::ALIGNMENT - 1) & !(Self::ALIGNMENT - 1);
    *i = start + Self::SIZE;
    start
  }

  fn write_aligned(&self, buffer: &mut [u8]);
}

impl Shared for bool {
  const ALIGNMENT: usize = u32::ALIGNMENT;
  const SIZE: usize = u32::ALIGNMENT;

  fn write_aligned(&self, buffer: &mut [u8]) {
    (*self as u32).write_aligned(buffer);
  }
}

impl Shared for f32 {
  const ALIGNMENT: usize = 4;
  const SIZE: usize = 4;

  fn write_aligned(&self, buffer: &mut [u8]) {
    buffer.copy_from_slice(&self.to_le_bytes());
  }
}

impl Shared for u32 {
  const ALIGNMENT: usize = 4;
  const SIZE: usize = 4;

  fn write_aligned(&self, buffer: &mut [u8]) {
    buffer.copy_from_slice(&self.to_le_bytes());
  }
}

impl Shared for Field {
  const ALIGNMENT: usize = u32::ALIGNMENT;
  const SIZE: usize = u32::SIZE;

  fn write_aligned(&self, buffer: &mut [u8]) {
    (*self as u32).write_aligned(buffer);
  }
}

impl Shared for Vec2f {
  const ALIGNMENT: usize = 8;
  const SIZE: usize = 8;

  fn write_aligned(&self, buffer: &mut [u8]) {
    self.x.write_aligned(&mut buffer[0..f32::SIZE]);
    self.y.write_aligned(&mut buffer[4..4 + f32::SIZE]);
  }
}
