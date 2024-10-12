use {
  self::renderer::Renderer,
  anyhow::Context,
  std::{backtrace::BacktraceStatus, borrow::Cow, process},
  wgpu::{
    Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, Instance,
    Limits, LoadOp, MemoryHints, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
    TextureViewDescriptor, VertexState,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
  },
};

type Result<T = ()> = anyhow::Result<T>;

mod renderer;

fn report(result: Result) {
  if let Err(error) = result {
    eprintln!("error: {error}");

    let backtrace = error.backtrace();

    if let BacktraceStatus::Captured = backtrace.status() {
      eprintln!("{}", backtrace);
    }

    process::exit(1);
  }
}

#[derive(Default)]
struct App {
  frame: u64,
  window: Option<Window>,
  renderer: Option<Renderer>,
}

impl App {
  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      eprintln!("Initializing window…");

      // todo:
      // - error handling?
      self.window = Some(
        event_loop
          .create_window(WindowAttributes::default().with_title("x"))
          .unwrap(),
      );

      assert!(self.renderer.is_none());

      // todo:
      // - this use of async is probably fucked
      self.renderer = Some(pollster::block_on(Renderer::new(self.window())).unwrap());

      eprintln!("Done initializing window…");
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match event {
      WindowEvent::RedrawRequested => {
        self.frame += 1;
        eprintln!("redraw {}", self.frame);
        self.window().request_redraw();
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      //   WindowEvent::Resized(size) => self.resize(size),
      _ => {}
    }
    eprintln!("window_event");
  }
}

fn run() -> Result<()> {
  env_logger::init();

  let event_loop = EventLoop::new()?;

  let mut app = App::default();

  event_loop.run_app(&mut app)?;

  // event_loop.run(|event, target| {
  //   report(renderer.handle_event(event, target));
  //   window.request_redraw();
  // })?;

  Ok(())
}

fn main() {
  report(run());
}
