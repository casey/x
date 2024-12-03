use super::*;

pub(crate) struct App {
  error: Option<anyhow::Error>,
  filters: Vec<Filter>,
  options: Options,
  renderer: Option<Renderer>,
  window: Option<Arc<Window>>,
}

impl App {
  pub(crate) fn new(options: Options) -> Self {
    Self {
      error: None,
      filters: Vec::new(),
      options,
      renderer: None,
      window: None,
    }
  }

  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }

  fn renderer(&mut self) -> &mut Renderer {
    self.renderer.as_mut().unwrap()
  }

  pub(crate) fn error(self) -> Option<anyhow::Error> {
    self.error
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
      WindowEvent::KeyboardInput { event, .. } => match event.logical_key {
        Key::Character(c) => {
          if event.state == ElementState::Pressed {
            match c.as_str() {
              "a" => self.filters.push(Filter { field: Field::All }),
              "c" => self.filters.push(Filter {
                field: Field::Circle,
              }),
              "x" => self.filters.push(Filter { field: Field::X }),
              _ => {}
            }
          }
        }
        Key::Named(NamedKey::Backspace) => {
          self.filters.pop();
        }
        _ => {}
      },
      WindowEvent::RedrawRequested => {
        if let Err(err) = self.renderer.as_mut().unwrap().render(&self.filters) {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
        self.window().request_redraw();
      }
      WindowEvent::Resized(size) => {
        self.renderer().resize(size);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
