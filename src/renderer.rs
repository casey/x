use super::*;

const UNIFORM_BUFFER_SIZE: u32 = 8;

struct Uniforms {
  field: Field,
  resolution: f32,
}

struct Target {
  bind_group: BindGroup,
  texture_view: TextureView,
}

pub struct Renderer {
  bind_group_layout: BindGroupLayout,
  config: SurfaceConfiguration,
  device: Device,
  frame: u64,
  initial_target: Target,
  queue: Queue,
  render_pipeline: RenderPipeline,
  sampler: Sampler,
  surface: Surface<'static>,
  targets: [Target; 2],
  texture_format: TextureFormat,
  uniform_buffer: Buffer,
  uniform_buffer_stride: u32,
}

impl Renderer {
  fn target(
    bind_group_layout: &BindGroupLayout,
    config: &SurfaceConfiguration,
    device: &Device,
    sampler: &Sampler,
    texture_format: TextureFormat,
    uniform_buffer: &Buffer,
  ) -> Target {
    let texture = device.create_texture(&TextureDescriptor {
      label: label!(),
      size: Extent3d {
        width: config.width,
        height: config.height,
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

    Target {
      bind_group,
      texture_view,
    }
  }

  pub async fn new(window: Arc<Window>) -> Result<Self> {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = Instance::default();

    let surface = instance.create_surface(window)?;

    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .context("failed to find an appropriate adapter")?;

    let (device, queue) = adapter
      .request_device(
        &DeviceDescriptor {
          label: label!(),
          required_features: Features::empty(),
          required_limits: Limits::default(),
          memory_hints: MemoryHints::Performance,
        },
        None,
      )
      .await
      .context("failed to create device")?;

    let texture_format = surface.get_capabilities(&adapter).formats[0];

    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let config = surface
      .get_default_config(&adapter, size.width, size.height)
      .context("failed to get default config")?;

    surface.configure(&device, &config);

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          count: None,
          ty: BindingType::Buffer {
            has_dynamic_offset: true,
            min_binding_size: Some(u64::from(UNIFORM_BUFFER_SIZE).try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: 1,
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: 2,
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    });

    let sampler = device.create_sampler(&SamplerDescriptor::default());

    let alignment = device.limits().min_uniform_buffer_offset_alignment;
    let padding = (alignment - UNIFORM_BUFFER_SIZE % alignment) % alignment;
    let uniform_buffer_stride = UNIFORM_BUFFER_SIZE + padding;

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
      label: label!(),
      mapped_at_creation: false,
      size: device.limits().max_buffer_size,
      usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      bind_group_layouts: &[&bind_group_layout],
      label: label!(),
      push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &shader,
        targets: &[Some(texture_format.into())],
      }),
      label: label!(),
      layout: Some(&pipeline_layout),
      multisample: MultisampleState::default(),
      multiview: None,
      primitive: PrimitiveState::default(),
      vertex: VertexState {
        buffers: &[],
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("vertex"),
        module: &shader,
      },
    });

    let initial_target = Self::target(
      &bind_group_layout,
      &config,
      &device,
      &sampler,
      texture_format,
      &uniform_buffer,
    );

    let targets = [
      Self::target(
        &bind_group_layout,
        &config,
        &device,
        &sampler,
        texture_format,
        &uniform_buffer,
      ),
      Self::target(
        &bind_group_layout,
        &config,
        &device,
        &sampler,
        texture_format,
        &uniform_buffer,
      ),
    ];

    Ok(Renderer {
      bind_group_layout,
      config,
      device,
      frame: 0,
      initial_target,
      queue,
      render_pipeline,
      sampler,
      surface,
      targets,
      texture_format,
      uniform_buffer,
      uniform_buffer_stride,
    })
  }

  fn write_uniform_buffer(&mut self, uniforms: &[Uniforms]) {
    if uniforms.is_empty() {
      return;
    }

    let size = u64::from(self.uniform_buffer_stride) * u64::try_from(uniforms.len()).unwrap();
    let mut buffer = self
      .queue
      .write_buffer_with(&self.uniform_buffer, 0, size.try_into().unwrap())
      .unwrap();

    for (uniform, dst) in uniforms
      .iter()
      .zip(buffer.chunks_mut(self.uniform_buffer_stride.try_into().unwrap()))
    {
      let Uniforms { field, resolution } = uniform;
      dst.write(&field.value()).write(&resolution.value());
    }
  }

  pub(crate) fn render(&mut self, filters: &[Filter]) -> Result {
    let filters = if filters.is_empty() {
      &[Filter { field: Field::None }]
    } else {
      filters
    };

    let resolution = self.config.width.max(self.config.height) as f32;

    let mut uniforms = Vec::new();

    for filter in filters {
      uniforms.push(Uniforms {
        field: filter.field,
        resolution,
      });
    }

    self.write_uniform_buffer(&uniforms);

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    let mut source = 0;
    let mut destination = 1;

    for i in 0..uniforms.len() {
      let view = if i == uniforms.len() - 1 {
        &frame.texture.create_view(&TextureViewDescriptor::default())
      } else {
        &self.targets[destination].texture_view
      };

      let source_target = if i == 0 {
        &self.initial_target
      } else {
        &self.targets[source]
      };

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[Some(RenderPassColorAttachment {
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
          },
          resolve_target: None,
          view,
        })],
        depth_stencil_attachment: None,
        label: label!(),
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      pass.set_bind_group(
        0,
        Some(&source_target.bind_group),
        &[self.uniform_buffer_stride * u32::try_from(i).unwrap()],
      );
      pass.set_pipeline(&self.render_pipeline);
      pass.draw(0..3, 0..1);

      (source, destination) = (destination, source);
    }

    self.queue.submit([encoder.finish()]);

    frame.present();

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
    self.config.width = size.width.max(1);
    self.config.height = size.height.max(1);
    self.surface.configure(&self.device, &self.config);
    for target in self.targets.iter_mut() {
      *target = Self::target(
        &self.bind_group_layout,
        &self.config,
        &self.device,
        &self.sampler,
        self.texture_format,
        &self.uniform_buffer,
      );
    }
  }
}
