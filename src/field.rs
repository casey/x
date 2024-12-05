#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub(crate) enum Field {
  #[default]
  All,
  Circle,
  None,
  X,
}
