use super::*;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

static BUFFER_SIZE: AtomicU32 = AtomicU32::new(0);

#[derive(Default)]
pub(crate) struct Uniforms {
  pub(crate) color: Matrix4,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) fit: bool,
  pub(crate) image_alpha: f32,
  pub(crate) offset: Vector2,
  pub(crate) position: Matrix3,
  pub(crate) repeat: bool,
  pub(crate) resolution: Vector2,
  pub(crate) source_alpha: f32,
  pub(crate) source_offset: Vector2,
  pub(crate) tiling: u32,
}

impl Uniforms {
  pub(crate) fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = 0;
    self.color.write(dst, &mut i, &mut a);
    self.coordinates.write(dst, &mut i, &mut a);
    self.field.write(dst, &mut i, &mut a);
    self.fit.write(dst, &mut i, &mut a);
    self.image_alpha.write(dst, &mut i, &mut a);
    self.offset.write(dst, &mut i, &mut a);
    self.position.write(dst, &mut i, &mut a);
    self.repeat.write(dst, &mut i, &mut a);
    self.resolution.write(dst, &mut i, &mut a);
    self.source_alpha.write(dst, &mut i, &mut a);
    self.source_offset.write(dst, &mut i, &mut a);
    self.tiling.write(dst, &mut i, &mut a);
    pad(i, a)
  }

  pub(crate) fn buffer_size() -> u32 {
    let buffer_size = BUFFER_SIZE.load(atomic::Ordering::Relaxed);

    if buffer_size != 0 {
      return buffer_size;
    }

    let mut buffer = vec![0; 1 * MIB];
    let buffer_size = Uniforms::default().write(&mut buffer).try_into().unwrap();
    BUFFER_SIZE.store(buffer_size, atomic::Ordering::Relaxed);
    buffer_size
  }
}
