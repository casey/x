use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Program {
  Highwaystar,
  Top,
  X,
}

impl Program {
  pub(crate) fn filters(self) -> Vec<Filter> {
    match self {
      Self::Highwaystar => Chain::default()
        .invert()
        .circle()
        .scale(2.0)
        .times(8)
        .into(),
      Self::Top => Chain::default().invert().top().push().into(),
      Self::X => Chain::default()
        .invert()
        .x()
        .push()
        .clear()
        .scale(2.0)
        .push()
        .into(),
    }
  }
}
