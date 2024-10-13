use super::*;

#[derive(Debug)]
pub(crate) enum Event {
  Thread(JoinHandle<Result>),
}
