use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Tiling {
  pub(crate) height: u32,
  pub(crate) size: u32,
  pub(crate) width: u32,
}

impl Tiling {
  pub(crate) fn image_read(&self, filters: u32) -> bool {
    if self.size == 1 {
      filters % 2 == 0
    } else {
      true
    }
  }

  pub(crate) fn offset(&self, filter: u32) -> Vec2f {
    if self.size == 1 {
      return Vec2f::new(0.0, 0.0);
    }

    let col = filter % self.size;
    let row = filter / self.size;

    Vec2f::new((self.width * col) as f32, (self.height * row) as f32)
  }

  pub(crate) fn resolution(&self) -> Vec2f {
    Vec2f::new(self.width as f32, self.height as f32)
  }

  pub(crate) fn set_viewport(&self, render_pass: &mut RenderPass, filter: u32) {
    if self.size == 1 {
      return;
    }

    let col = filter % self.size;
    let row = filter / self.size;

    render_pass.set_viewport(
      (col * self.width) as f32,
      (row * self.height) as f32,
      self.width as f32,
      self.height as f32,
      0.0,
      0.0,
    );
  }

  pub(crate) fn source_offset(&self, filter: u32) -> Vec2f {
    if self.size == 1 {
      return Vec2f::new(0.0, 0.0);
    }

    let Some(filter) = filter.checked_sub(1) else {
      return Vec2f::new(0.0, 0.0);
    };

    let row = filter / self.size;
    let col = filter % self.size;

    Vec2f::new(col as f32 / self.size as f32, row as f32 / self.size as f32)
  }

  pub(crate) fn source_read(&self, filters: u32) -> bool {
    if self.size == 1 {
      filters % 2 == 1
    } else {
      true
    }
  }
}
