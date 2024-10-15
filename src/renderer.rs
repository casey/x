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
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState,
  },
};

// todo:
// - get screenshots back

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

const UNIFORM_BUFFER_SIZE: u32 = 8;
const SCREENSHOT_RESOLUTION: u32 = 4096;

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
  proxy: EventLoopProxy<Event>,
  queue: Queue,
  render_pipeline: RenderPipeline,
  sampler: Sampler,
  surface: Surface<'static>,
  targets: Vec<Target>,
  texture_format: TextureFormat,
  uniform_buffer: Buffer,
  uniform_buffer_stride: u32,
}

impl Renderer {
  fn target(&self) -> Target {
    let texture = self.device.create_texture(&TextureDescriptor {
      label: label!(),
      size: Extent3d {
        width: self.config.width,
        height: self.config.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: self.texture_format,
      usage: TextureUsages::RENDER_ATTACHMENT
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST,
      view_formats: &[self.texture_format],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&texture_view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(&self.sampler),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &self.uniform_buffer,
            offset: 0,
            size: Some(u64::from(UNIFORM_BUFFER_SIZE).try_into().unwrap()),
          }),
        },
      ],
      label: label!(),
    });

    Target {
      bind_group,
      texture_view,
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
            has_dynamic_offset: true,
            min_binding_size: Some(u64::from(UNIFORM_BUFFER_SIZE).try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    });

    let sampler = device.create_sampler(&SamplerDescriptor::default());

    let min_uniform_buffer_offset_alignment = device.limits().min_uniform_buffer_offset_alignment;

    let data = u32::from(UNIFORM_BUFFER_SIZE);
    let alignment = min_uniform_buffer_offset_alignment;
    let padding = (alignment - data % alignment) % alignment;
    let uniform_buffer_stride = data + padding;

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

    let mut renderer = Renderer {
      bind_group_layout,
      config,
      device,
      frame: 0,
      proxy,
      queue,
      render_pipeline,
      sampler,
      surface,
      targets: Vec::with_capacity(2),
      texture_format,
      uniform_buffer,
      uniform_buffer_stride,
    };

    renderer.targets.push(renderer.target());
    renderer.targets.push(renderer.target());
    renderer.targets.push(renderer.target());

    Ok(renderer)
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

  pub(crate) fn render(&mut self) -> Result {
    let resolution = self.config.width.max(self.config.height) as f32;

    let filters = [
      Filter { field: Field::All },
      Filter { field: Field::All },
      Filter { field: Field::X },
    ];

    let mut uniforms = Vec::new();

    for filter in &filters {
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
        &self.targets[2]
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

  #[allow(unused)]
  pub(crate) fn render_old(&mut self) -> Result {
    let frame = self
      .surface
      .get_current_texture()
      .context("failed to acquire next swap chain texture")?;

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let screenshot_buffer = if self.frame == 0 {
      let buffer = self.device.create_buffer(&BufferDescriptor {
        label: label!(),
        size: (SCREENSHOT_RESOLUTION * SCREENSHOT_RESOLUTION * 4).into(),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
      });

      let texture = self.device.create_texture(&TextureDescriptor {
        label: label!(),
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
          label: label!(),
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
    self.targets[0] = self.target();
    self.targets[1] = self.target();
    self.targets[2] = self.target();
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
