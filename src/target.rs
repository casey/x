use super::*;

pub(crate) struct Target {
  pub(crate) bind_group: BindGroup,
  pub(crate) texture_view: TextureView,
}

impl Target {
  pub(crate) fn new(
    bind_group_layout: &BindGroupLayout,
    device: &Device,
    resolution: u32,
    sampler: &Sampler,
    texture_format: TextureFormat,
    uniform_buffer: &Buffer,
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
      usage: TextureUsages::RENDER_ATTACHMENT
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST,
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
            size: Some(u64::from(UNIFORM_BUFFER_SIZE).try_into().unwrap()),
          }),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::TextureView(&texture_view),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Sampler(sampler),
        },
      ],
      label: label!(),
    });

    Self {
      bind_group,
      texture_view,
    }
  }
}
