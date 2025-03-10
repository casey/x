use super::*;

#[derive(Clone, Copy, Default, EnumIter, IntoStaticStr)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Circle,
  Frequencies,
  #[default]
  None,
  Samples,
  Top,
  X,
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
