use super::*;

pub(crate) struct Frame {
  pub(crate) fps: Option<f64>,
  pub(crate) number: u64,
  pub(crate) filters: usize,
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
