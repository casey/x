use super::*;

pub(crate) struct App {
  error: Option<Error>,
  filters: Vec<Filter>,
  #[allow(unused)]
  input: cpal::Stream,
  makro: Vec<Key>,
  options: Options,
  recording: Option<Vec<Key>>,
  renderer: Option<Renderer>,
  sample_queue: Arc<Mutex<VecDeque<f32>>>,
  samples: Vec<f32>,
  window: Option<Arc<Window>>,
}

impl App {
  pub(crate) fn error(self) -> Option<Error> {
    self.error
  }

  pub(crate) fn new(options: Options) -> Result<Self> {
    let device = cpal::default_host()
      .default_input_device()
      .context(error::DefaultAudioInputDevice)?;

    let supported_stream_config = device
      .supported_input_configs()
      .context(error::SupportedStreamConfigs)?
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::SupportedStreamConfig)?
      .with_max_sample_rate();

    let buffer_size = match supported_stream_config.buffer_size() {
      SupportedBufferSize::Range { min, .. } => {
        log::info!("input audio buffer size: {min}");
        Some(*min)
      }
      SupportedBufferSize::Unknown => {
        log::info!("input audio buffer size: unknown");
        None
      }
    };

    let mut stream_config = supported_stream_config.config();

    if let Some(buffer_size) = buffer_size {
      stream_config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
    }

    let sample_queue = Arc::new(Mutex::new(VecDeque::new()));

    let write = sample_queue.clone();

    let input = device
      .build_input_stream(
        &stream_config,
        move |data: &[f32], _| {
          write.lock().unwrap().extend(data);
        },
        move |err| {
          eprintln!("audio error: {err}");
        },
        None,
      )
      .context(error::BuildAudioInputStream)?;

    Ok(Self {
      error: None,
      filters: Vec::new(),
      input,
      makro: Vec::new(),
      options,
      recording: None,
      renderer: None,
      sample_queue,
      samples: Vec::new(),
      window: None,
    })
  }

  fn press(&mut self, key: Key) {
    let mut capture = true;

    match key {
      Key::Character(ref c) => match c.as_str() {
        "@" => {
          for key in self.makro.clone() {
            self.press(key);
          }
          capture = false;
        }
        "a" => self.filters.push(Filter {
          color: invert_color(),
          field: Field::All,
          wrap: self.options.wrap,
          ..default()
        }),
        "c" => self.filters.push(Filter {
          color: invert_color(),
          field: Field::Circle,
          wrap: self.options.wrap,
          ..default()
        }),
        "d" => self.filters.push(Filter {
          coordinates: true,
          wrap: self.options.wrap,
          ..default()
        }),
        "f" => {
          self.options.fit = !self.options.fit;
        }
        "n" => self.filters.push(Filter {
          field: Field::None,
          wrap: self.options.wrap,
          ..default()
        }),
        "q" => {
          if let Some(recording) = self.recording.take() {
            self.makro = recording;
          } else {
            self.recording = Some(Vec::new());
          }
          capture = false;
        }
        "r" => {
          self.options.repeat = !self.options.repeat;
        }
        "s" => self.filters.push(Filter {
          color: invert_color(),
          field: Field::Samples,
          wrap: self.options.wrap,
          ..default()
        }),
        "t" => {
          self.options.tile = !self.options.tile;
        }
        "w" => {
          self.options.wrap = !self.options.wrap;
        }
        "x" => self.filters.push(Filter {
          color: invert_color(),
          field: Field::X,
          wrap: self.options.wrap,
          ..default()
        }),
        "z" => self.filters.push(Filter {
          position: Mat3f::new_scaling(2.0),
          wrap: self.options.wrap,
          ..default()
        }),
        _ => {}
      },
      Key::Named(key) => match key {
        NamedKey::Backspace => {
          self.filters.pop();
        }
        NamedKey::ArrowLeft => {
          self.filters.push(Filter {
            position: Mat3f::new_rotation(-0.01),
            ..default()
          });
        }
        NamedKey::ArrowRight => {
          self.filters.push(Filter {
            position: Mat3f::new_rotation(0.01),
            ..default()
          });
        }
        _ => {}
      },
      _ => {}
    }

    if capture {
      if let Some(recording) = &mut self.recording {
        recording.push(key);
      }
    }
  }

  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      assert!(self.renderer.is_none());

      let window = match event_loop
        .create_window(
          WindowAttributes::default()
            .with_inner_size(PhysicalSize {
              width: 1024,
              height: 1024,
            })
            .with_min_inner_size(PhysicalSize {
              width: 256,
              height: 256,
            })
            .with_title("x"),
        )
        .context(error::CreateWindow)
      {
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      self.window = Some(window.clone());

      let renderer = match pollster::block_on(Renderer::new(&self.options, window)) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      self.renderer = Some(renderer);
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    if self.renderer.is_none() {
      event_loop.exit();
      return;
    }

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event.logical_key);
      }
      WindowEvent::RedrawRequested => {
        self.samples.clear();
        self
          .samples
          .extend(self.sample_queue.lock().unwrap().drain(..));

        if let Err(err) =
          self
            .renderer
            .as_mut()
            .unwrap()
            .render(&self.options, &self.filters, &self.samples)
        {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
        self.window().request_redraw();
      }
      WindowEvent::Resized(size) => {
        self.renderer.as_mut().unwrap().resize(&self.options, size);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
