use {
  self::{
    app::App, field::Field, filter::Filter, renderer::Renderer, shared::Shared, slice_ext::SliceExt,
  },
  anyhow::Context,
  std::{backtrace::BacktraceStatus, process, sync::Arc, thread::JoinHandle},
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
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
mod field;
mod filter;
mod renderer;
mod shared;
mod slice_ext;

fn run() -> Result<()> {
  env_logger::init();

  let mut app = App::default();

  EventLoop::with_user_event().build()?.run_app(&mut app)?;

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
