#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Circle,
  Frequencies,
  #[default]
  None,
  Samples,
  X,
}
