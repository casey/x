#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  #[allow(unused)]
  None,
  X,
}
