use super::*;

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Bottom,
  Circle,
  Frequencies,
  None,
  Samples,
  Top,
  X,
}

impl Default for Field {
  fn default() -> Self {
    Field::None
  }
}

impl Field {
  pub(crate) fn constant(self) -> String {
    format!("FIELD_{}", self.name().to_uppercase())
  }

  pub(crate) fn function(self) -> String {
    format!("field_{}", self.name().to_lowercase())
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
