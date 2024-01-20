use super::*;

pub struct Renderer<'a> {
  config: SurfaceConfiguration,
  device: Device,
  surface: Surface<'a>,
  render_pipeline: RenderPipeline,
  queue: Queue,
  window: &'a Window,
}

impl<'a> Renderer<'a> {
  pub async fn new(window: &'a Window) -> Self {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = Instance::default();

    let surface = instance.create_surface(window).unwrap();
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
      .request_device(
        &DeviceDescriptor {
          label: None,
          required_features: Features::empty(),
          required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        },
        None,
      )
      .await
      .expect("Failed to create device");

    let shader = device.create_shader_module(ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader,
        entry_point: "vertex",
        buffers: &[],
      },
      fragment: Some(FragmentState {
        module: &shader,
        entry_point: "fragment",
        targets: &[Some(swapchain_format.into())],
      }),
      primitive: PrimitiveState::default(),
      depth_stencil: None,
      multisample: MultisampleState::default(),
      multiview: None,
    });

    let config = surface
      .get_default_config(&adapter, size.width, size.height)
      .unwrap();
    surface.configure(&device, &config);

    Renderer {
      config,
      device,
      queue,
      render_pipeline,
      surface,
      window,
    }
  }

  pub fn handle_event(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) {
    {
      match event {
        Event::WindowEvent { event, .. } => {
          match event {
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CloseRequested => self.close(target),
            _ => {}
          };
        }
        _ => {}
      }
    }
  }

  fn close(&self, target: &EventLoopWindowTarget<()>) {
    target.exit();
  }

  fn redraw(&self) {
    let frame = self
      .surface
      .get_current_texture()
      .expect("Failed to acquire next swap chain texture");
    let view = frame.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor { label: None });
    {
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
  }

  fn resize(&mut self, size: PhysicalSize<u32>) {
    self.config.width = size.width.max(1);
    self.config.height = size.height.max(1);
    self.surface.configure(&self.device, &self.config);
    self.window.request_redraw();
  }
}
