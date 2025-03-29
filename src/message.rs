#[derive(Clone, Copy, Debug)]
pub(crate) struct Message {
  pub(crate) channel: midly::num::u4,
  pub(crate) key: midly::num::u7,
  pub(crate) on: bool,
  pub(crate) timestamp: u64,
  pub(crate) velocity: midly::num::u7,
}
