use super::*;

pub struct Renderer {
  bind_group_layout: BindGroupLayout,
  bindings: Option<Bindings>,
  config: SurfaceConfiguration,
  device: Device,
  error_channel: std::sync::mpsc::Receiver<wgpu::Error>,
  font: Font,
  frame: u64,
  frame_times: VecDeque<Instant>,
  frequencies: Texture,
  frequency_view: TextureView,
  overlay_renderer: vello::Renderer,
  queue: Queue,
  render_pipeline: RenderPipeline,
  resolution: u32,
  sample_view: TextureView,
  sampler: Sampler,
  samples: Texture,
  size: Vec2u,
  surface: Surface<'static>,
  texture_format: TextureFormat,
  uniform_buffer: Buffer,
  uniform_buffer_size: u32,
  uniform_buffer_stride: u32,
}

impl Renderer {
  pub async fn new(options: &Options, window: Arc<Window>) -> Result<Self> {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = Instance::default();

    let surface = instance
      .create_surface(window)
      .context(error::CreateSurface)?;

    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .context(error::Adapter)?;

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
      .context(error::Device)?;

    let (tx, error_channel) = mpsc::channel();

    device.on_uncaptured_error(Box::new(move |error| tx.send(error).unwrap()));

    let texture_format = surface.get_capabilities(&adapter).formats[0];

    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let config = surface
      .get_default_config(&adapter, size.width, size.height)
      .context(error::DefaultConfig)?;

    surface.configure(&device, &config);

    let uniform_buffer_size = {
      let mut buffer = vec![0; MIB];
      u32::try_from(Uniforms::default().write(&mut buffer)).unwrap()
    };

    let bind_group_layout = Self::bind_group_layout(&device, uniform_buffer_size);

    let sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::Repeat,
      address_mode_v: AddressMode::Repeat,
      ..default()
    });

    let limits = device.limits();

    let alignment = limits.min_uniform_buffer_offset_alignment;
    let padding = (alignment - uniform_buffer_size % alignment) % alignment;
    let uniform_buffer_stride = uniform_buffer_size + padding;

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

    let samples = device.create_texture(&TextureDescriptor {
      dimension: TextureDimension::D1,
      format: TextureFormat::R32Float,
      label: label!(),
      mip_level_count: 1,
      sample_count: 1,
      size: Extent3d {
        depth_or_array_layers: 1,
        height: 1,
        width: limits.max_texture_dimension_1d,
      },
      usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
      view_formats: &[TextureFormat::R32Float],
    });

    let sample_view = samples.create_view(&TextureViewDescriptor::default());

    let frequencies = device.create_texture(&TextureDescriptor {
      dimension: TextureDimension::D1,
      format: TextureFormat::R32Float,
      label: label!(),
      mip_level_count: 1,
      sample_count: 1,
      size: Extent3d {
        depth_or_array_layers: 1,
        height: 1,
        width: limits.max_texture_dimension_1d,
      },
      usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
      view_formats: &[TextureFormat::R32Float],
    });

    let frequency_view = frequencies.create_view(&TextureViewDescriptor::default());

    let resolution = options.resolution(size);

    let overlay_renderer = vello::Renderer::new(
      &device,
      vello::RendererOptions {
        antialiasing_support: vello::AaSupport::all(),
        num_init_threads: Some(1.try_into().unwrap()),
        surface_format: None,
        use_cpu: false,
      },
    )
    .context(error::CreateOverlayRenderer)?;

    let mut renderer = Renderer {
      bind_group_layout,
      bindings: None,
      config,
      device,
      error_channel,
      font: load_font(FONT)?,
      frame: 0,
      frame_times: VecDeque::with_capacity(100),
      frequencies,
      frequency_view,
      overlay_renderer,
      queue,
      render_pipeline,
      resolution,
      sample_view,
      sampler,
      samples,
      size: Vec2u::new(size.width, size.height),
      surface,
      texture_format,
      uniform_buffer,
      uniform_buffer_size,
      uniform_buffer_stride,
    };

    renderer.resize(options, size);

    Ok(renderer)
  }

  fn bind_group(
    &self,
    frequencies: &TextureView,
    back: &TextureView,
    samples: &TextureView,
    front: &TextureView,
  ) -> BindGroup {
    let mut i = 0;
    let mut binding = || {
      let binding = i;
      i += 1;
      binding
    };
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(back),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Sampler(&self.sampler),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(frequencies),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(front),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Sampler(&self.sampler),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(samples),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &self.uniform_buffer,
            offset: 0,
            size: Some(u64::from(self.uniform_buffer_size).try_into().unwrap()),
          }),
        },
      ],
      label: label!(),
    })
  }

  fn bind_group_layout(device: &Device, uniform_buffer_size: u32) -> BindGroupLayout {
    let mut i = 0;
    let mut binding = || {
      let binding = i;
      i += 1;
      binding
    };
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Buffer {
            has_dynamic_offset: true,
            min_binding_size: Some(u64::from(uniform_buffer_size).try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    })
  }

  fn bindings(&self) -> &Bindings {
    self.bindings.as_ref().unwrap()
  }

  fn draw(
    &self,
    bind_group: &BindGroup,
    encoder: &mut CommandEncoder,
    tiling: Option<(Tiling, u32)>,
    uniform: u32,
    view: &TextureView,
  ) {
    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
      color_attachments: &[Some(RenderPassColorAttachment {
        ops: Operations {
          load: LoadOp::Load,
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

    pass.set_bind_group(0, Some(bind_group), &[self.uniform_buffer_stride * uniform]);

    pass.set_pipeline(&self.render_pipeline);

    if let Some((tiling, filter)) = tiling {
      tiling.set_viewport(&mut pass, filter);
    }

    pass.draw(0..3, 0..1);
  }

  pub(crate) fn render(
    &mut self,
    options: &Options,
    analyzer: &Analyzer,
    filters: &[Filter],
  ) -> Result {
    match self.error_channel.try_recv() {
      Ok(error) => return Err(error::Validation.into_error(error)),
      Err(mpsc::TryRecvError::Empty) => {}
      Err(mpsc::TryRecvError::Disconnected) => panic!("error channel disconnected"),
    }

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

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tiling_size = if options.tile {
      (filters.len().max(1) as f64).sqrt().ceil() as u32
    } else {
      1
    };

    let tiling = Tiling {
      resolution: self.resolution / tiling_size,
      size: tiling_size,
    };

    let sample_count = analyzer
      .samples()
      .len()
      .min(self.samples.width().into_usize());
    let samples = &analyzer.samples()[..sample_count];
    let sample_range = sample_count as f32 / self.samples.width() as f32;
    self.write_texture(samples, &self.samples);

    let frequency_count = analyzer
      .frequencies()
      .len()
      .min(self.frequencies.width().into_usize());
    let frequencies = &analyzer.frequencies()[..frequency_count];
    let frequency_range = frequency_count as f32 / self.frequencies.width() as f32;
    self.write_texture(frequencies, &self.frequencies);

    let filter_count = u32::try_from(filters.len()).unwrap();

    for (i, filter) in filters.iter().enumerate() {
      let i = u32::try_from(i).unwrap();
      uniforms.push(Uniforms {
        back_read: false,
        color: filter.color,
        coordinates: filter.coordinates,
        field: filter.field,
        filters: filter_count,
        fit: false,
        frequency_range,
        front_offset: tiling.source_offset(i),
        index: i,
        offset: tiling.offset(i),
        position: filter.position,
        repeat: false,
        resolution: tiling.resolution(),
        sample_range,
        front_read: true,
        tiling: tiling.size,
        wrap: filter.wrap,
      });
    }

    uniforms.push(Uniforms {
      back_read: tiling.back_read(filter_count),
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::None,
      filters: filter_count,
      fit: options.fit,
      frequency_range,
      front_offset: Vec2f::new(0.0, 0.0),
      index: filter_count,
      offset: Vec2f::default(),
      position: Mat3f::identity(),
      repeat: options.repeat,
      resolution: Vec2f::new(self.resolution as f32, self.resolution as f32),
      sample_range,
      front_read: tiling.front_read(filter_count),
      tiling: 1,
      wrap: false,
    });

    uniforms.push(Uniforms {
      back_read: true,
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::None,
      filters: filter_count,
      fit: options.fit,
      frequency_range,
      front_offset: Vec2f::new(0.0, 0.0),
      index: filter_count,
      offset: Vec2f::default(),
      position: Mat3f::identity(),
      repeat: options.repeat,
      resolution: Vec2f::new(self.size.x as f32, self.size.y as f32),
      sample_range,
      front_read: true,
      tiling: 1,
      wrap: false,
    });

    self.write_uniform_buffer(&uniforms);

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let frame = self
      .surface
      .get_current_texture()
      .context(error::CurrentTexture)?;

    for target in &self.bindings().targets {
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
    for i in 0..filters.len() {
      let i = u32::try_from(i).unwrap();
      self.draw(
        &self.bindings().targets[source].bind_group,
        &mut encoder,
        Some((tiling, i)),
        i,
        &self.bindings().targets[destination].texture_view,
      );
      (source, destination) = (destination, source);
    }

    self.draw(
      &self.bindings().tiling_bind_group,
      &mut encoder,
      None,
      filter_count,
      &self.bindings().tiling,
    );

    self.render_overlay()?;

    self.draw(
      &self.bindings().overlay_bind_group,
      &mut encoder,
      None,
      filter_count + 1,
      &frame.texture.create_view(&TextureViewDescriptor::default()),
    );

    self.queue.submit([encoder.finish()]);

    frame.present();

    info!(
      "{}",
      Frame {
        filters: filters.len(),
        fps,
        number: self.frame,
      }
    );

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn render_overlay(&mut self) -> Result {
    use {
      kurbo::Affine,
      peniko::{Brush, Color, Fill},
      skrifa::{instance::Size, raw::FileRef},
      vello::{AaConfig, Glyph, RenderParams, Scene},
    };

    let mut scene = Scene::new();

    let font_size = 64.0;

    let file = FileRef::new(self.font.data.as_ref()).context(error::FontRead)?;

    let font = match file {
      FileRef::Collection(c) => c.get(self.font.index).context(error::FontRead)?,
      FileRef::Font(f) => f,
    };

    let charmap = font.charmap();
    let settings: [(&str, f32); 0] = [];
    let location = font.axes().location(settings.iter().copied());
    let metrics = font.glyph_metrics(Size::new(font_size), &location);
    let mut x = 0.0;

    let glyphs = "hello world"
      .chars()
      .map(|character| {
        let id = charmap
          .map(character)
          .context(error::FontGlyph { character })?;

        let glyph = Glyph {
          id: id.into(),
          x,
          y: 0.0,
        };

        x += metrics.advance_width(id).unwrap_or_default();

        Ok(glyph)
      })
      .collect::<Result<Vec<Glyph>>>()?;

    scene
      .draw_glyphs(&self.font)
      .font_size(font_size)
      .brush(&Brush::Solid(Color::WHITE))
      .transform(Affine::translate((110.0, 700.0)))
      .glyph_transform(None)
      .draw(Fill::NonZero, glyphs.into_iter());

    self
      .overlay_renderer
      .render_to_texture(
        &self.device,
        &self.queue,
        &scene,
        &self.bindings.as_ref().unwrap().overlay,
        &RenderParams {
          base_color: Color::TRANSPARENT,
          width: self.resolution,
          height: self.resolution,
          antialiasing_method: AaConfig::Msaa16,
        },
      )
      .context(error::RenderOverlay)?;

    Ok(())
  }

  pub(crate) fn resize(&mut self, options: &Options, size: PhysicalSize<u32>) {
    self.config.height = size.height.max(1);
    self.config.width = size.width.max(1);
    self.resolution = options.resolution(size);
    self.size = Vec2u::new(size.width, size.height);
    self.surface.configure(&self.device, &self.config);

    let tiling = self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: self.texture_format,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: self.resolution,
          width: self.resolution,
        },
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[self.texture_format],
      })
      .create_view(&TextureViewDescriptor::default());

    let targets = [self.target(&tiling), self.target(&tiling)];

    let tiling_bind_group = self.bind_group(
      &self.frequency_view,
      &targets[0].texture_view,
      &self.sample_view,
      &targets[1].texture_view,
    );

    let overlay_format = vello::low_level::ImageFormat::Rgba8.to_wgpu();
    let overlay = self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: overlay_format,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: self.resolution,
          width: self.resolution,
        },
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[overlay_format],
      })
      .create_view(&TextureViewDescriptor::default());

    let overlay_bind_group =
      self.bind_group(&self.frequency_view, &tiling, &self.sample_view, &overlay);

    self.bindings = Some(Bindings {
      overlay,
      overlay_bind_group,
      targets,
      tiling,
      tiling_bind_group,
    });
  }

  fn target(&self, back: &TextureView) -> Target {
    let texture = self.device.create_texture(&TextureDescriptor {
      dimension: TextureDimension::D2,
      format: self.texture_format,
      label: label!(),
      mip_level_count: 1,
      sample_count: 1,
      size: Extent3d {
        depth_or_array_layers: 1,
        height: self.resolution,
        width: self.resolution,
      },
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[self.texture_format],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let bind_group = self.bind_group(&self.frequency_view, back, &self.sample_view, &texture_view);

    Target {
      bind_group,
      texture,
      texture_view,
    }
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
      .zip(buffer.chunks_mut(self.uniform_buffer_stride.into_usize()))
    {
      uniforms.write(dst);
    }
  }

  fn write_texture(&self, data: &[f32], destination: &Texture) {
    self.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: destination,
        mip_level: 0,
        origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect: TextureAspect::All,
      },
      &data
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<u8>>(),
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: None,
        rows_per_image: None,
      },
      Extent3d {
        width: data.len().try_into().unwrap(),
        height: 1,
        depth_or_array_layers: 1,
      },
    );
  }
}
