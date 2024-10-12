use {
  self::renderer::Renderer,
  anyhow::Context,
  std::{backtrace::BacktraceStatus, borrow::Cow, process, sync::Arc},
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
    event::WindowEvent,
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
  window: Option<Arc<Window>>,
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
      self.window = Some(Arc::new(
        event_loop
          .create_window(WindowAttributes::default().with_title("x"))
          .unwrap(),
      ));

      assert!(self.renderer.is_none());

      // todo:
      // - this use of async is probably fucked
      self.renderer =
        Some(pollster::block_on(Renderer::new(self.window.as_ref().unwrap().clone())).unwrap());

      eprintln!("Done initializing window…");
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match event {
      WindowEvent::RedrawRequested => {
        self.frame += 1;
        self.renderer.as_ref().unwrap().redraw().unwrap();
        eprintln!("redraw {}", self.frame);
        self.window().request_redraw();
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::Resized(size) => {
        self.renderer.as_mut().unwrap().resize(size);
        self.window().request_redraw();
      }
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

  Ok(())
}

fn main() {
  report(run());
}
