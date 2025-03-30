use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
  Button(bool),
  Encoder(Parameter),
}
