use {
  super::*,
  wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, Color,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, FragmentState,
    ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, Instance, Limits, LoadOp, Maintain,
    MapMode, MemoryHints, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
  },
};

// todo:
// - get screenshots back
// - i think i need to always clear the first source, or use an empty texture

const SCREENSHOT_RESOLUTION: u32 = 4096;
const SAMPLE_UNIFORM_BUFFER_SIZE: u64 = 4;

struct Target {
  bind_group: BindGroup,
  texture: Texture,
}

pub struct Renderer {
  config: SurfaceConfiguration,
  device: Device,
  frame: u64,
  proxy: EventLoopProxy<Event>,
  queue: Queue,
  render_pipeline: RenderPipeline,
  sample_group: BindGroup,
  sample_pipeline: RenderPipeline,
  sample_uniform_buffer: Buffer,
  sample_bind_group_layout: BindGroupLayout,
  surface: Surface<'static>,
  texture: Texture,
  sampler: Sampler,
  targets: Vec<Target>,
  texture_format: TextureFormat,
}

impl Renderer {
  fn target(&self) -> Target {
    let texture = self.device.create_texture(&TextureDescriptor {
      label: None,
      size: Extent3d {
        width: self.config.width,
        height: self.config.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: self.texture_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[self.texture_format],
    });

    let view = texture.create_view(&TextureViewDescriptor::default());

    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.sample_bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(&self.sampler),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &self.sample_uniform_buffer,
            offset: 0,
            size: None,
          }),
        },
      ],
      label: Some("sample bind group"),
    });

    Target {
      bind_group,
      texture,
    }
  }

  pub async fn new(window: Arc<Window>, proxy: EventLoopProxy<Event>) -> Result<Self> {
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
          label: Some("device"),
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

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &shader,
        targets: &[Some(texture_format.into())],
      }),
      label: Some("render pipeline"),
      layout: None,
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

    let sample = device.create_shader_module(include_wgsl!("sample.wgsl"));

    let config = surface
      .get_default_config(&adapter, size.width, size.height)
      .context("failed to get default config")?;

    surface.configure(&device, &config);

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: 1,
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: 2,
          count: None,
          ty: BindingType::Buffer {
            has_dynamic_offset: false,
            min_binding_size: Some(SAMPLE_UNIFORM_BUFFER_SIZE.try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: Some("sample bind group layout"),
    });

    let sampler = device.create_sampler(&SamplerDescriptor::default());

    let sample_uniform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("sample uniform buffer"),
      size: SAMPLE_UNIFORM_BUFFER_SIZE,
      usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
      mapped_at_creation: false,
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("sample pipeline layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let sample_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &sample,
        targets: &[Some(texture_format.into())],
      }),
      label: Some("sample pipeline"),
      layout: Some(&pipeline_layout),
      multisample: MultisampleState::default(),
      multiview: None,
      primitive: PrimitiveState::default(),
      vertex: VertexState {
        buffers: &[],
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("vertex"),
        module: &sample,
      },
    });

    let texture = Self::create_texture(&device, &config, texture_format);

    let view = texture.create_view(&TextureViewDescriptor::default());

    let sample_group = device.create_bind_group(&BindGroupDescriptor {
      layout: &bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(&sampler),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &sample_uniform_buffer,
            offset: 0,
            size: None,
          }),
        },
      ],
      label: Some("sample bind group"),
    });

    let mut renderer = Renderer {
      config,
      device,
      frame: 0,
      proxy,
      queue,
      render_pipeline,
      sample_group,
      sample_pipeline,
      sample_uniform_buffer,
      sample_bind_group_layout: bind_group_layout,
      sampler,
      surface,
      texture,
      texture_format,
      targets: Vec::with_capacity(2),
    };

    renderer.targets.push(renderer.target());
    renderer.targets.push(renderer.target());

    Ok(renderer)
  }

  pub(crate) fn render_two(&mut self) -> Result {
    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let resolution = self.config.width.max(self.config.height) as f32;

    self
      .queue
      .write_buffer(&self.sample_uniform_buffer, 0, &resolution.to_le_bytes());

    let filters = vec![(), ()];

    let mut output = 0;

    for (i, _filter) in filters.into_iter().enumerate() {
      let source = i % 2;
      let destination = (i + 1) % 2;

      let view = self.targets[destination]
        .texture
        .create_view(&TextureViewDescriptor {
          format: Some(self.texture_format),
          ..TextureViewDescriptor::default()
        });

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some(&format!("filter {i} render pass")),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });

      pass.set_bind_group(0, Some(&self.targets[source].bind_group), &[]);
      pass.set_pipeline(&self.sample_pipeline);
      pass.draw(0..3, 0..1);

      output = destination;
    }

    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    {
      let view = frame.texture.create_view(&TextureViewDescriptor {
        format: Some(self.texture_format),
        ..TextureViewDescriptor::default()
      });

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("final render pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });

      pass.set_bind_group(0, Some(&self.targets[output].bind_group), &[]);
      pass.set_pipeline(&self.sample_pipeline);
      pass.draw(0..3, 0..1);
    }

    self.queue.submit(Some(encoder.finish()));

    frame.present();

    self.frame += 1;

    Ok(())
  }

  #[allow(unused)]
  pub(crate) fn render(&mut self) -> Result {
    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let screenshot_buffer = if self.frame == 0 {
      let buffer = self.device.create_buffer(&BufferDescriptor {
        label: None,
        size: (SCREENSHOT_RESOLUTION * SCREENSHOT_RESOLUTION * 4).into(),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
      });

      let texture = self.device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
          width: SCREENSHOT_RESOLUTION,
          height: SCREENSHOT_RESOLUTION,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: self.texture_format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[self.texture_format],
      });

      {
        let view = texture.create_view(&TextureViewDescriptor::default());
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
          label: None,
          color_attachments: &[Some(RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: Operations {
              load: LoadOp::Clear(Color::BLACK),
              store: StoreOp::Store,
            },
          })],
          depth_stencil_attachment: None,
          timestamp_writes: None,
          occlusion_query_set: None,
        });
        pass.set_pipeline(&self.render_pipeline);
        pass.draw(0..3, 0..1);
      }

      encoder.copy_texture_to_buffer(
        ImageCopyTexture {
          texture: &texture,
          mip_level: 0,
          origin: Origin3d::ZERO,
          aspect: TextureAspect::All,
        },
        ImageCopyBuffer {
          buffer: &buffer,
          layout: ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(SCREENSHOT_RESOLUTION * 4),
            rows_per_image: Some(SCREENSHOT_RESOLUTION),
          },
        },
        Extent3d {
          width: SCREENSHOT_RESOLUTION,
          height: SCREENSHOT_RESOLUTION,
          depth_or_array_layers: 1,
        },
      );

      Some(buffer)
    } else {
      None
    };

    // todo:
    // - begin with empty textures a and b
    // - for each filter, render
    // - copy last texture to screen
    //
    // - how to calculate texture coordinates?
    // - texture should be size of screen
    // - can calculate coordinates in vertex shader or fragment shader

    {
      let view = self.texture.create_view(&TextureViewDescriptor::default());
      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
      pass.set_pipeline(&self.render_pipeline);
      pass.draw(0..3, 0..1);
    }

    {
      let resolution = self.config.width.max(self.config.height) as f32;

      self
        .queue
        .write_buffer(&self.sample_uniform_buffer, 0, &resolution.to_le_bytes());

      let view = frame.texture.create_view(&TextureViewDescriptor {
        format: Some(self.texture_format),
        ..TextureViewDescriptor::default()
      });

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("sample render pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });

      pass.set_pipeline(&self.sample_pipeline);
      pass.set_bind_group(0, Some(&self.sample_group), &[]);
      pass.draw(0..3, 0..1);
    }

    self.queue.submit(Some(encoder.finish()));

    if let Some(buffer) = screenshot_buffer {
      self.save_screenshot(buffer);
    }

    frame.present();

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
    self.config.width = size.width.max(1);
    self.config.height = size.height.max(1);
    self.surface.configure(&self.device, &self.config);
    self.texture = Self::create_texture(&self.device, &self.config, self.texture_format);
    self.targets[0] = self.target();
    self.targets[1] = self.target();
  }

  fn create_texture(
    device: &Device,
    config: &SurfaceConfiguration,
    texture_format: TextureFormat,
  ) -> Texture {
    device.create_texture(&TextureDescriptor {
      label: None,
      size: Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: texture_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[texture_format],
    })
  }

  fn save_screenshot(&self, buffer: Buffer) {
    let join_handle = std::thread::spawn(move || {
      let buffer_slice = buffer.slice(..);
      let (tx, rx) = std::sync::mpsc::channel();
      buffer_slice.map_async(MapMode::Read, move |r| tx.send(r).unwrap());

      rx.recv().unwrap()?;

      let screenshot = Image::new(
        SCREENSHOT_RESOLUTION,
        SCREENSHOT_RESOLUTION,
        buffer_slice.get_mapped_range().as_ref().into(),
      );

      buffer.unmap();

      let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

      let path = dirs::home_dir()
        .unwrap()
        .join(format!("Dropbox/x/{timestamp}.png"));

      screenshot.write(&path)?;

      Ok(())
    });

    self.proxy.send_event(Event::Thread(join_handle)).unwrap();

    self.device.poll(Maintain::wait()).panic_on_timeout();
  }
}
