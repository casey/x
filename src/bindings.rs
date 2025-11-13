use super::*;

pub(crate) struct Bindings {
  pub(crate) captures: Arc<Mutex<Vec<Buffer>>>,
  pub(crate) overlay_bind_group: BindGroup,
  pub(crate) overlay_view: TextureView,
  pub(crate) targets: [Target; 2],
  pub(crate) tiling_bind_group: BindGroup,
  pub(crate) tiling_view: TextureView,
}
