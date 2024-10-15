use {
  self::{
    app::App, event::Event, field::Field, filter::Filter, image::Image, renderer::Renderer,
    shared::Shared, slice_ext::SliceExt,
  },
  anyhow::Context,
  std::{
    backtrace::BacktraceStatus,
    fs::File,
    path::Path,
    process,
    sync::Arc,
    thread::JoinHandle,
    time::{SystemTime, UNIX_EPOCH},
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
  },
};

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

type Result<T = ()> = anyhow::Result<T>;

mod app;
mod event;
mod field;
mod filter;
mod image;
mod renderer;
mod shared;
mod slice_ext;

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
