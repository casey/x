use super::*;

pub(crate) struct Target {
  pub(crate) bind_group: BindGroup,
  pub(crate) texture: Texture,
  pub(crate) texture_view: TextureView,
}
