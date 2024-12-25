#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Circle,
  #[default]
  None,
  Samples,
  X,
}
