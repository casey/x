use super::*;

pub(crate) struct Filter {
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) position: Mat3f,
  pub(crate) wrap: bool,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::default(),
      position: Mat3f::identity(),
      wrap: false,
    }
  }
}
