use super::*;

pub(crate) trait Shared {
  const ALIGNMENT: usize;
  const SIZE: usize;

  fn slot(i: &mut usize) -> usize {
    let start = pad(*i, Self::ALIGNMENT);
    *i = start + Self::SIZE;
    start
  }

  fn write(&self, buffer: &mut [u8], i: &mut usize, alignment: &mut usize) {
    *alignment = (*alignment).max(Self::ALIGNMENT);
    let start = Self::slot(i);
    self.write_aligned(&mut buffer[start..*i]);
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

impl Shared for Matrix3 {
  const ALIGNMENT: usize = 16;
  const SIZE: usize = 48;

  fn write_aligned(&self, buffer: &mut [u8]) {
    for (column, buffer) in self.column_iter().zip(buffer.chunks_mut(f32::SIZE * 4)) {
      for (scalar, buffer) in column.as_slice().iter().zip(buffer.chunks_mut(f32::SIZE)) {
        scalar.write_aligned(buffer);
      }
    }
  }
}

impl Shared for Matrix4 {
  const ALIGNMENT: usize = 16;
  const SIZE: usize = 64;

  fn write_aligned(&self, buffer: &mut [u8]) {
    for (scalar, buffer) in self.as_slice().iter().zip(buffer.chunks_mut(f32::SIZE)) {
      scalar.write_aligned(buffer);
    }
  }
}

impl Shared for Vector2 {
  const ALIGNMENT: usize = 8;
  const SIZE: usize = 8;

  fn write_aligned(&self, buffer: &mut [u8]) {
    for (scalar, buffer) in self.as_slice().iter().zip(buffer.chunks_mut(f32::SIZE)) {
      scalar.write_aligned(buffer);
    }
  }
}
