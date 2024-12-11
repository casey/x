use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to get adapter"))]
  Adapter,
  #[snafu(display("failed to create surface"))]
  CreateSurface { source: wgpu::CreateSurfaceError },
  #[snafu(display("failed to create window"))]
  CreateWindow { source: winit::error::OsError },
  #[snafu(display("failed to get current texture"))]
  CurrentTexture { source: wgpu::SurfaceError },
  #[snafu(display("failed to get default config"))]
  DefaultConfig,
  #[snafu(display("failed to get device"))]
  Device { source: wgpu::RequestDeviceError },
  #[snafu(display("failed to build event loop"))]
  EventLoopBuild {
    source: winit::error::EventLoopError,
  },
  #[snafu(display("failed to run app"))]
  RunApp {
    source: winit::error::EventLoopError,
  },
  #[snafu(display("validation failed"))]
  Validation { source: wgpu::Error },
}
