use super::*;

#[derive(Default)]
pub(crate) struct Test {
  failures: u32,
}

pub(crate) fn run() {
  let mut test = Test::default();

  EventLoop::with_user_event()
    .build()
    .unwrap()
    .run_app(&mut test)
    .unwrap();

  if test.failures > 0 {
    eprintln!("{} failures", test.failures);
  }
}

impl ApplicationHandler for Test {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let options = Options::default();

    let window = event_loop
      .create_window(
        WindowAttributes::default()
          .with_inner_size(PhysicalSize {
            width: 4095,
            height: 4095,
          })
          .with_min_inner_size(PhysicalSize {
            width: 256,
            height: 256,
          })
          .with_title("test"),
      )
      .unwrap();

    let analyzer = Analyzer::new();

    let mut renderer = pollster::block_on(Renderer::new(&options, window.into())).unwrap();
    let mut actual = Image::default();
    let tests = Path::new("programs");

    for program in Program::value_variants() {
      let name = program.to_possible_value().unwrap().get_name().to_owned();

      let expected = Image::load(&tests.join(format!("{name}.png"))).unwrap();

      pollster::block_on(renderer.render(&options, &analyzer, &program.state())).unwrap();

      pollster::block_on(renderer.capture(&mut actual)).unwrap();

      if actual != expected {
        self.failures += 1;
        eprintln!("image mismatch {name}");
      }
    }

    event_loop.exit();
  }

  fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
}
