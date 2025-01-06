use super::*;

pub(crate) struct Bindings {
  pub(crate) bind_group: BindGroup,
  pub(crate) composite_bind_group: BindGroup,
  pub(crate) image: TextureView,
  pub(crate) overlay: TextureView,
  pub(crate) targets: [Target; 2],
}
