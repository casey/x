use super::*;

pub(crate) struct App {
  analyzer: Analyzer,
  capture: Image,
  error: Option<Error>,
  filters: Vec<Filter>,
  makro: Vec<Key>,
  options: Options,
  recording: Option<Vec<Key>>,
  renderer: Option<Renderer>,
  stream: Box<dyn Stream>,
  window: Option<Arc<Window>>,
}

impl App {
  pub(crate) fn new(options: Options) -> Result<Self> {
    let stream: Box<dyn Stream> = if let Some(track) = &options.track {
      Box::new(Track::load(track)?)
    } else {
      Box::new(Input::new()?)
    };

    Ok(Self {
      analyzer: Analyzer::new(),
      capture: Image::default(),
      error: None,
      filters: options.program.map(Program::filters).unwrap_or_default(),
      makro: Vec::new(),
      options,
      recording: None,
      renderer: None,
      stream,
      window: None,
    })
  }

  fn capture(&mut self) -> Result {
    pollster::block_on(self.renderer.as_mut().unwrap().capture(&mut self.capture))?;
    self.capture.save("capture.png".as_ref())?;
    Ok(())
  }

  pub(crate) fn error(self) -> Option<Error> {
    self.error
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key) {
    let mut capture = true;

    match key {
      Key::Character(ref c) => match c.as_str() {
        ">" => {
          if let Err(err) = self.capture() {
            self.error = Some(err);
            event_loop.exit();
          }
        }
        "@" => {
          for key in self.makro.clone() {
            self.press(event_loop, key);
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
        "l" => self.filters.push(Filter {
          color: invert_color(),
          field: Field::Frequencies,
          wrap: self.options.wrap,
          ..default()
        }),
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

  fn redraw(&mut self, event_loop: &ActiveEventLoop) {
    self.analyzer.update(self.stream.as_mut());

    if let Err(err) =
      self
        .renderer
        .as_mut()
        .unwrap()
        .render(&self.options, &self.analyzer, &self.filters)
    {
      self.error = Some(err);
      event_loop.exit();
      return;
    }

    self.window().request_redraw();
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
        self.press(event_loop, event.logical_key);
      }
      WindowEvent::RedrawRequested => {
        self.redraw(event_loop);
      }
      WindowEvent::Resized(size) => {
        self.renderer.as_mut().unwrap().resize(&self.options, size);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
