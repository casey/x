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
  pub(crate) fn state(self) -> State {
    match self {
      Self::Bottom => State::default().invert().bottom().push(),
      Self::Circle => State::default().invert().circle().push(),
      Self::Frequencies | Self::Hello => State::default()
        .db(-40)
        .text(Some(Text {
          size: 0.05,
          string: "hello world".into(),
          x: 0.10,
          y: -0.10,
        }))
        .invert()
        .frequencies()
        .push(),
      Self::Highwaystar => State::default().invert_r().circle().scale(2.0).times(8),
      Self::Middle => State::default().invert().top().push().bottom().push(),
      Self::Rip => State::default().invert().top().push().samples().push(),
      Self::Top => State::default().invert().top().push(),
      Self::X => State::default().invert().x().push(),
    }
  }
}
