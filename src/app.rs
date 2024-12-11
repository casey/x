use super::*;

pub(crate) struct App {
  error: Option<anyhow::Error>,
  filters: Vec<Filter>,
  makro: Vec<Key>,
  options: Options,
  recording: Option<Vec<Key>>,
  renderer: Option<Renderer>,
  window: Option<Arc<Window>>,
}

fn invert() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

impl App {
  pub(crate) fn error(self) -> Option<anyhow::Error> {
    self.error
  }

  pub(crate) fn new(options: Options) -> Self {
    Self {
      error: None,
      filters: Vec::new(),
      makro: Vec::new(),
      options,
      recording: None,
      renderer: None,
      window: None,
    }
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
          color: invert(),
          field: Field::All,
          wrap: self.options.wrap,
          ..default()
        }),
        "c" => self.filters.push(Filter {
          color: invert(),
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
        "t" => {
          self.options.tile = !self.options.tile;
        }
        "w" => {
          self.options.wrap = !self.options.wrap;
        }
        "x" => self.filters.push(Filter {
          color: invert(),
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

      let window = match event_loop.create_window(
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
      ) {
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.error = Some(err.into());
          event_loop.exit();
          return;
        }
      };

      let renderer = match pollster::block_on(Renderer::new(self.options.clone(), window.clone())) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      self.window = Some(window);

      self.renderer = Some(renderer);
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event.logical_key);
      }
      WindowEvent::RedrawRequested => {
        if let Err(err) = self
          .renderer
          .as_mut()
          .unwrap()
          .render(&self.options, &self.filters)
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
