use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Program {
  Bottom,
  Circle,
  Frequencies,
  Hello,
  Highwaystar,
  Middle,
  Rip,
  Top,
  X,
}

impl Program {
  pub(crate) fn db(self) -> i64 {
    match self {
      Self::Hello => -40,
      _ => 0,
    }
  }

  pub(crate) fn filters(self) -> Vec<Filter> {
    match self {
      Self::Bottom => Chain::default().invert().bottom().push(),
      Self::Circle => Chain::default().invert().circle().push(),
      Self::Frequencies | Self::Hello => Chain::default().invert().frequencies().push(),
      Self::Highwaystar => Chain::default().invert_r().circle().scale(2.0).times(8),
      Self::Middle => Chain::default().invert().top().push().bottom().push(),
      Self::Rip => Chain::default().invert().top().push().samples().push(),
      Self::Top => Chain::default().invert().top().push(),
      Self::X => Chain::default().invert().x().push(),
    }
    .into()
  }

  pub(crate) fn text(self) -> Option<Text> {
    match self {
      Self::Hello => Some(Text {
        size: 64.0,
        string: "hello world".into(),
        x: 0.10,
        y: -0.10,
      }),
      _ => None,
    }
  }
}
