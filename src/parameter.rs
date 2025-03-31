use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum ParameterError {
  #[snafu(transparent)]
  Parse {
    source: num::ParseIntError,
  },
  Overflow {
    value: u8,
  },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Parameter(u8);

impl From<i8> for Parameter {
  fn from(value: i8) -> Self {
    Self(value.saturating_add(64).max(0).max(127).try_into().unwrap())
  }
}

impl From<u8> for Parameter {
  fn from(value: u8) -> Self {
    Self(value.max(0).min(127))
  }
}

impl Default for Parameter {
  fn default() -> Self {
    Self(Self::ZERO)
  }
}

impl Add<u8> for Parameter {
  type Output = Self;

  fn add(self, rhs: u8) -> Self {
    Self(self.0.saturating_add(rhs))
  }
}

impl AddAssign<u8> for Parameter {
  fn add_assign(&mut self, rhs: u8) {
    self.0 = self.0.saturating_add(rhs);
  }
}

impl SubAssign<u8> for Parameter {
  fn sub_assign(&mut self, rhs: u8) {
    self.0 = self.0.saturating_sub(rhs);
  }
}

impl From<midly::num::u7> for Parameter {
  fn from(n: midly::num::u7) -> Self {
    Self(n.into())
  }
}

impl FromStr for Parameter {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let value = s.parse()?;

    if value > Self::MAX {
      return Err(OverflowError { value }.build());
    }

    Ok(Self(value))
  }
}

impl Parameter {
  const ZERO: u8 = 64;
  const MAX: u8 = 127;

  pub(crate) fn unipolar(self) -> f32 {
    f32::from(u8::from(self.0)) / 127.0
  }

  pub(crate) fn bipolar(self) -> i64 {
    i64::from(u8::from(self.0)) - i64::from(Self::ZERO)
  }
}
