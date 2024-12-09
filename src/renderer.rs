use super::*;

pub struct Renderer {
  bind_group_layout: BindGroupLayout,
  config: SurfaceConfiguration,
  device: Device,
  final_target: Target,
  frame: u64,
  frame_times: VecDeque<Instant>,
  image_view: TextureView,
  queue: Queue,
  render_pipeline: RenderPipeline,
  resolution: u32,
  sampler: Sampler,
  size: PhysicalSize<u32>,
  surface: Surface<'static>,
  targets: [Target; 2],
  texture_format: TextureFormat,
  uniform_buffer: Buffer,
  uniform_buffer_stride: u32,
}

impl Renderer {
  pub async fn new(options: Options, window: Arc<Window>) -> Result<Self> {
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
          required_features: Features::CLEAR_TEXTURE,
          required_limits: Limits::default(),
          memory_hints: MemoryHints::Performance,
        },
        None,
      )
      .await
      .context("failed to create device")?;

    // todo:
    // device.on_uncaptured_error(Box::new(|error| {}));

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
            min_binding_size: Some(u64::from(Uniforms::buffer_size()).try_into().unwrap()),
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
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: 3,
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    });

    let sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::MirrorRepeat,
      address_mode_v: AddressMode::MirrorRepeat,
      ..default()
    });

    let limits = device.limits();

    let alignment = limits.min_uniform_buffer_offset_alignment;
    let padding = (alignment - Uniforms::buffer_size() % alignment) % alignment;
    let uniform_buffer_stride = Uniforms::buffer_size() + padding;

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
      label: label!(),
      mapped_at_creation: false,
      size: limits.max_buffer_size,
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

    let resolution = options.resolution(size);

    let image_texture = device.create_texture(&TextureDescriptor {
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
      usage: TextureUsages::TEXTURE_BINDING,
      view_formats: &[texture_format],
    });

    let image_view = image_texture.create_view(&TextureViewDescriptor::default());

    let target = || {
      Target::new(
        &bind_group_layout,
        &device,
        &image_view,
        resolution,
        &sampler,
        texture_format,
        &uniform_buffer,
        None,
      )
    };

    let targets = [target(), target()];

    let final_target = Target::new(
      &bind_group_layout,
      &device,
      &targets[0].texture_view,
      resolution,
      &sampler,
      texture_format,
      &uniform_buffer,
      Some(&targets[1].texture_view),
    );

    Ok(Renderer {
      bind_group_layout,
      config,
      device,
      final_target,
      frame: 0,
      frame_times: VecDeque::with_capacity(100),
      image_view,
      queue,
      render_pipeline,
      resolution,
      sampler,
      size,
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

    for (uniforms, dst) in uniforms
      .iter()
      .zip(buffer.chunks_mut(self.uniform_buffer_stride.try_into().unwrap()))
    {
      uniforms.write(dst);
    }
  }

  pub(crate) fn render(&mut self, options: &Options, filters: &[Filter]) -> Result {
    if self.frame_times.len() == self.frame_times.capacity() {
      self.frame_times.pop_front();
    }

    let now = Instant::now();

    self.frame_times.push_back(now);

    let fps = if self.frame_times.len() >= 2 {
      let elapsed = *self.frame_times.back().unwrap() - *self.frame_times.front().unwrap();
      Some(1000.0 / (elapsed.as_millis() as f64 / self.frame_times.len() as f64))
    } else {
      None
    };

    let mut uniforms = Vec::new();

    #[derive(Clone, Copy)]
    struct Tiling {
      height: u32,
      size: u32,
      width: u32,
    }

    let tiling = if options.tile && !filters.is_empty() {
      let size = (filters.len() as f64).sqrt().ceil() as u32;
      let side = self.resolution / size;
      Some(Tiling {
        height: self.resolution / size,
        size,
        width: self.resolution / size,
      })
    } else {
      None
    };

    for (i, filter) in filters.iter().enumerate() {
      let resolution = if let Some(tiling) = tiling {
        Vector2::new(tiling.width as f32, tiling.height as f32)
      } else {
        Vector2::new(self.resolution as f32, self.resolution as f32)
      };

      let offset = if let Some(tiling) = tiling {
        let col = i as u32 % tiling.size;
        let row = i as u32 / tiling.size;
        Vector2::new((tiling.width * col) as f32, (tiling.height * row) as f32)
      } else {
        Vector2::new(0.0, 0.0)
      };

      dbg!(offset);

      let source_offset = if let Some(tiling) = tiling {
        if let Some(i) = i.checked_sub(1) {
          let row = i as u32 / tiling.size;
          let col = i as u32 % tiling.size;
          Vector2::new(
            col as f32 / tiling.size as f32,
            row as f32 / tiling.size as f32,
          )
        } else {
          Vector2::new(0.0, 0.0)
        }
      } else {
        Vector2::new(0.0, 0.0)
      };

      dbg!(source_offset);

      uniforms.push(Uniforms {
        color: filter.color,
        coordinates: filter.coordinates,
        field: filter.field,
        filters: filters.len().try_into().unwrap(),
        fit: false,
        image_alpha: 0.0,
        index: i.try_into().unwrap(),
        offset,
        position: filter.position,
        repeat: false,
        resolution,
        source_alpha: 1.0,
        source_offset,
        tiling: if let Some(tiling) = tiling {
          tiling.size
        } else {
          1
        },
      });
    }

    uniforms.push(Uniforms {
      color: Matrix4::identity(),
      coordinates: false,
      field: Field::None,
      filters: filters.len().try_into().unwrap(),
      // todo: change to false
      fit: options.fit,
      offset: Vector2::default(),
      position: Matrix3::identity(),
      // todo: change to false
      repeat: options.repeat,
      resolution: Vector2::new(self.size.width as f32, self.size.height as f32),
      source_offset: Vector2::new(0.0, 0.0),
      source_alpha: if tiling.is_some() {
        1.0
      } else {
        (filters.len() % 2) as f32
      },
      image_alpha: if tiling.is_some() {
        1.0
      } else {
        ((filters.len() + 1) % 2) as f32
      },
      index: filters.len().try_into().unwrap(),
      tiling: 1,
    });

    self.write_uniform_buffer(&uniforms);

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    for target in &self.targets {
      encoder.clear_texture(
        &target.texture,
        &ImageSubresourceRange {
          aspect: TextureAspect::All,
          base_mip_level: 0,
          mip_level_count: None,
          base_array_layer: 0,
          array_layer_count: None,
        },
      );
    }

    let mut source = 0;
    let mut destination = 1;
    let mut uniforms = 0;

    for i in 0..filters.len() {
      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[Some(RenderPassColorAttachment {
          ops: Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
          },
          resolve_target: None,
          view: &self.targets[destination].texture_view,
        })],
        depth_stencil_attachment: None,
        label: label!(),
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      pass.set_bind_group(
        0,
        Some(&self.targets[source].bind_group),
        &[self.uniform_buffer_stride * uniforms],
      );

      pass.set_pipeline(&self.render_pipeline);

      if let Some(tiling) = tiling {
        let i = u32::try_from(i).unwrap();
        let col = i % tiling.size;
        let row = i / tiling.size;
        pass.set_viewport(
          (col * tiling.width) as f32,
          (row * tiling.height) as f32,
          tiling.width as f32,
          tiling.height as f32,
          0.0,
          0.0,
        );
      }

      pass.draw(0..3, 0..1);

      uniforms += 1;

      (source, destination) = (destination, source);
    }

    {
      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[Some(RenderPassColorAttachment {
          ops: Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
          },
          resolve_target: None,
          view: &frame.texture.create_view(&TextureViewDescriptor::default()),
        })],
        depth_stencil_attachment: None,
        label: label!(),
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      pass.set_bind_group(
        0,
        Some(&self.final_target.bind_group),
        &[self.uniform_buffer_stride * uniforms],
      );

      pass.set_pipeline(&self.render_pipeline);

      pass.draw(0..3, 0..1);
    }

    self.queue.submit([encoder.finish()]);

    frame.present();

    info!(
      "{}",
      Frame {
        number: self.frame,
        fps,
        filters: filters.len()
      }
    );

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn resize(&mut self, options: &Options, size: PhysicalSize<u32>) {
    self.config.height = size.height.max(1);
    self.config.width = size.width.max(1);
    self.resolution = options.resolution(size);
    self.size = size;
    self.surface.configure(&self.device, &self.config);

    let image_texture = self.device.create_texture(&TextureDescriptor {
      label: label!(),
      size: Extent3d {
        width: self.resolution,
        height: self.resolution,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: self.texture_format,
      usage: TextureUsages::TEXTURE_BINDING,
      view_formats: &[self.texture_format],
    });

    self.image_view = image_texture.create_view(&TextureViewDescriptor::default());

    for target in self.targets.iter_mut() {
      *target = Target::new(
        &self.bind_group_layout,
        &self.device,
        &self.image_view,
        self.resolution,
        &self.sampler,
        self.texture_format,
        &self.uniform_buffer,
        None,
      );
    }

    self.final_target = Target::new(
      &self.bind_group_layout,
      &self.device,
      &self.targets[0].texture_view,
      self.resolution,
      &self.sampler,
      self.texture_format,
      &self.uniform_buffer,
      Some(&self.targets[1].texture_view),
    );
  }
}
