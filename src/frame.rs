use super::*;

pub(crate) struct Frame {
  pub(crate) filters: usize,
  pub(crate) fps: Option<f32>,
  pub(crate) number: u64,
}

impl Display for Frame {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "frame {} · {}",
      self.number,
      Tally("filter", self.filters),
    )?;

    if let Some(fps) = self.fps {
      write!(f, " · {fps:.0} fps")?;
    }

    Ok(())
  }
}
