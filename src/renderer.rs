use super::*;

pub struct Renderer {
  config: SurfaceConfiguration,
  device: Device,
  frame: u64,
  queue: Queue,
  render_pipeline: RenderPipeline,
  surface: Surface<'static>,
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
          required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
          memory_hints: MemoryHints::Performance,
        },
        None,
      )
      .await
      .context("failed to create device")?;

    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());

    let swapchain_capabilities = surface.get_capabilities(&adapter);

    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &shader,
        targets: &[Some(swapchain_format.into())],
      }),
      label: None,
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
    })
  }

  pub(crate) fn render(&mut self) -> Result {
    eprintln!("rendering frame {}", self.frame);

    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    let view = frame.texture.create_view(&TextureViewDescriptor::default());

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    {
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
