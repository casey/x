use super::*;

pub(crate) struct Bindings {
  pub(crate) capture: Buffer,
  pub(crate) overlay: TextureView,
  pub(crate) overlay_bind_group: BindGroup,
  pub(crate) targets: [Target; 2],
  pub(crate) tiling: TextureView,
  pub(crate) tiling_bind_group: BindGroup,
  pub(crate) tiling_texture: Texture,
}
