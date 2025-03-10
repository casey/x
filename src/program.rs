use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Program {
  Circle,
  Highwaystar,
  Rip,
  Top,
  X,
}

impl Program {
  pub(crate) fn filters(self) -> Vec<Filter> {
    match self {
      Self::Circle => Chain::default().invert().circle().push(),
      Self::Highwaystar => Chain::default().invert().circle().scale(2.0).times(8),
      Self::Rip => Chain::default().invert().top().push().samples().push(),
      Self::Top => Chain::default().invert().top().push(),
      Self::X => Chain::default().invert().x().push(),
    }
    .into()
  }
}
