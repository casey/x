use {
  super::*,
  wgpu::{
    include_wgsl, Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Extent3d, Features, FragmentState, ImageCopyBuffer, ImageCopyTexture,
    ImageDataLayout, Instance, Limits, LoadOp, Maintain, MapMode, MemoryHints, MultisampleState,
    Operations, Origin3d, PipelineCompilationOptions, PowerPreference, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    VertexState,
  },
};

pub struct Renderer {
  config: SurfaceConfiguration,
  device: Device,
  frame: u64,
  queue: Queue,
  render_pipeline: RenderPipeline,
  surface: Surface<'static>,
  texture_format: TextureFormat,
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

    let texture_format = surface.get_capabilities(&adapter).formats[0];

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

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &shader,
        targets: &[Some(texture_format.into())],
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
      texture_format,
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

    let buffer = if self.frame == 0 {
      let buffer = self.device.create_buffer(&BufferDescriptor {
        label: None,
        size: self.subpixels().into(),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
      });

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
            bytes_per_row: Some(self.config.width * 4),
            rows_per_image: Some(self.config.height),
          },
        },
        Extent3d {
          width: self.config.width,
          height: self.config.height,
          depth_or_array_layers: 1,
        },
      );

      Some(buffer)
    } else {
      None
    };

    // render to frame
    {
      let view = frame.texture.create_view(&TextureViewDescriptor::default());

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

    self.queue.submit(Some(encoder.finish()));

    if let Some(buffer) = buffer {
      self.save_screenshot(self.config.width, self.config.height, buffer);
    }

    frame.present();

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
    self.config.width = size.width.max(1);
    self.config.height = size.height.max(1);
    self.surface.configure(&self.device, &self.config);
  }

  fn pixels(&self) -> u32 {
    self.config.width.checked_mul(self.config.height).unwrap()
  }

  fn subpixels(&self) -> u32 {
    self.pixels().checked_mul(4).unwrap()
  }

  fn save_screenshot(&self, width: u32, height: u32, buffer: Buffer) {
    std::thread::spawn(move || {
      let buffer_slice = buffer.slice(..);
      let (tx, rx) = std::sync::mpsc::channel();
      buffer_slice.map_async(MapMode::Read, move |r| tx.send(r).unwrap());

      rx.recv().unwrap().unwrap();

      let screenshot = Image::new(
        width,
        height,
        buffer_slice.get_mapped_range().as_ref().into(),
      );

      buffer.unmap();

      screenshot.write("screenshot.png".into()).unwrap();
    });

    self.device.poll(Maintain::wait()).panic_on_timeout();
  }
}
