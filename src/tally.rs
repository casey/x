use super::*;

pub(crate) struct Tally(pub(crate) &'static str, pub(crate) usize);

impl Display for Tally {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {}", self.1, self.0)?;
    if self.1 != 1 {
      write!(f, "s")?;
    }
    Ok(())
  }
}
