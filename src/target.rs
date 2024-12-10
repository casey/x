use super::*;

pub(crate) struct Target {
  pub(crate) bind_group: BindGroup,
  pub(crate) texture: Option<Texture>,
  pub(crate) texture_view: TextureView,
}

impl Target {
  pub(crate) fn new(
    bind_group_layout: &BindGroupLayout,
    device: &Device,
    image_view: &TextureView,
    resolution: u32,
    sampler: &Sampler,
    texture_format: TextureFormat,
    uniform_buffer: &Buffer,
    substitute_texture_view: Option<&TextureView>,
  ) -> Self {
    let texture = device.create_texture(&TextureDescriptor {
      label: label!(),
      size: Extent3d {
        width: resolution,
        height: resolution,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: texture_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[texture_format],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
      layout: bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: uniform_buffer,
            offset: 0,
            size: Some(u64::from(Uniforms::buffer_size()).try_into().unwrap()),
          }),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::TextureView(image_view),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::TextureView(substitute_texture_view.unwrap_or(&texture_view)),
        },
        BindGroupEntry {
          binding: 3,
          resource: BindingResource::Sampler(sampler),
        },
      ],
      label: label!(),
    });

    Self {
      bind_group,
      texture: Some(texture),
      texture_view,
    }
  }
}
