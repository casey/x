use super::*;

// todo:
// - render to texture
// - save screenshot
// - set up github CI
// - ping pong rendering

pub struct Renderer {
  config: SurfaceConfiguration,
  device: Device,
  frame: u64,
  queue: Queue,
  render_pipeline: RenderPipeline,
  surface: Surface<'static>,
  texture: Texture,
}

impl Renderer {
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
          label: None,
          required_features: Features::empty(),
          required_limits: Limits::default(),
          memory_hints: MemoryHints::Performance,
        },
        None,
      )
      .await
      .context("failed to create device")?;

    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let texture_format = surface.get_capabilities(&adapter).formats[0];

    let texture = device.create_texture(&TextureDescriptor {
      label: None,
      size: Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: texture_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
      view_formats: &[texture_format],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &shader,
        targets: &[Some(surface.get_capabilities(&adapter).formats[0].into())],
      }),
      label: None,
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

    let config = surface
      .get_default_config(&adapter, size.width, size.height)
      .context("failed to get default config")?;

    surface.configure(&device, &config);

    Ok(Renderer {
      config,
      device,
      frame: 0,
      queue,
      render_pipeline,
      surface,
      texture,
    })
  }

  pub(crate) fn render(&mut self) -> Result {
    eprintln!("rendering frame {}", self.frame);

    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    // todo: get rid of 4
    let mut data = Vec::<u8>::with_capacity(
      (self.config.width * self.config.height * 4)
        .try_into()
        .unwrap(),
    );

    let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: data.capacity().try_into().unwrap(),
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation: false,
    });

    // render to texture
    {
      let view = self
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::GREEN),
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
      wgpu::ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          // todo:
          // - this needs to be a multiple of 256?
          bytes_per_row: Some((self.config.width * 4).try_into().unwrap()),
          rows_per_image: Some(self.config.height),
        },
      },
      wgpu::Extent3d {
        width: self.config.width,
        height: self.config.height,
        depth_or_array_layers: 1,
      },
    );

    // render to frame
    {
      let view = frame.texture.create_view(&TextureViewDescriptor::default());

      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::GREEN),
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

    self.queue.submit(Some(encoder.finish()));

    frame.present();

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
    self.config.width = size.width.max(1);
    self.config.height = size.height.max(1);
    self.surface.configure(&self.device, &self.config);
  }
}
