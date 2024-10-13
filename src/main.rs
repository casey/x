use {
  self::{app::App, event::Event, image::Image, renderer::Renderer},
  anyhow::Context,
  camino::Utf8Path,
  std::{backtrace::BacktraceStatus, fs::File, process, sync::Arc, thread::JoinHandle},
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
  },
};

type Result<T = ()> = anyhow::Result<T>;

mod app;
mod event;
mod image;
mod renderer;

fn run() -> Result<()> {
  env_logger::init();

  let event_loop = EventLoop::with_user_event().build()?;

  let mut app = App::new(&event_loop);

  event_loop.run_app(&mut app)?;

  if let Some(err) = app.error() {
    return Err(err);
  }

  Ok(())
}

fn main() {
  if let Err(error) = run() {
    eprintln!("error: {error}");

    let backtrace = error.backtrace();

    if let BacktraceStatus::Captured = backtrace.status() {
      eprintln!("{}", backtrace);
    }

    process::exit(1);
  }
}
