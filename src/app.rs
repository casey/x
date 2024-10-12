use super::*;

#[derive(Default)]
pub(crate) struct App {
  error: Option<anyhow::Error>,
  renderer: Option<Renderer>,
  window: Option<Arc<Window>>,
}

impl App {
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

      let window = match event_loop.create_window(WindowAttributes::default().with_title("x")) {
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.error = Some(err.into());
          event_loop.exit();
          return;
        }
      };

      let renderer = match pollster::block_on(Renderer::new(window.clone())) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.error = Some(err.into());
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
      WindowEvent::RedrawRequested => {
        if let Err(err) = self.renderer().render() {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
        self.window().request_redraw();
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::Resized(size) => {
        self.renderer().resize(size);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
