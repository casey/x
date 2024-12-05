use super::*;

pub(crate) struct Filter {
  pub(crate) color: Matrix4,
  pub(crate) field: Field,
  pub(crate) position: Matrix3,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      color: Matrix4::identity(),
      field: Field::default(),
      position: Matrix3::identity(),
    }
  }
}
