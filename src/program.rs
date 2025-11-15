use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Program {
  All,
  Bottom,
  Circle,
  Frequencies,
  Hello,
  Highwaystar,
  Middle,
  None,
  RedX,
  Rip,
  Top,
  X,
}

impl Program {
  pub(crate) fn state(self) -> State {
    match self {
      Self::All => State::default().invert().all().push(),
      Self::Bottom => State::default().invert().bottom().push(),
      Self::Circle => State::default().invert().circle().push(),
      Self::Frequencies => State::default().invert().frequencies().push(),
      Self::Hello => State::default()
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
      Self::Highwaystar => State::default().invert().circle().scale(2.0).times(8),
      Self::Middle => State::default().invert().top().push().bottom().push(),
      Self::None => State::default(),
      Self::RedX => State::default().invert_r().x().push(),
      Self::Rip => State::default().invert().top().push().samples().push(),
      Self::Top => State::default().invert().top().push(),
      Self::X => State::default().invert().x().push(),
    }
  }
}
