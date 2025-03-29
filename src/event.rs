#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
  Button { press: bool },
  Encoder { value: u8 },
}
