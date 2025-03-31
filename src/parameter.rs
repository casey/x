use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum ParameterError {
  #[snafu(display("value less than minimum: {value}"))]
  NegativeOverflow { value: i8 },
  #[snafu(transparent)]
  Parse { source: num::ParseIntError },
  #[snafu(display("value greater than maximum: {value}"))]
  PositiveOverflow { value: i8 },
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Parameter(i8);

impl From<i8> for Parameter {
  fn from(value: i8) -> Self {
    Self(value.clamp(Self::MIN, Self::MAX))
  }
}

impl Add<i8> for Parameter {
  type Output = Self;

  fn add(self, rhs: i8) -> Self {
    self.0.saturating_add(rhs).into()
  }
}

impl AddAssign<i8> for Parameter {
  fn add_assign(&mut self, rhs: i8) {
    *self = self.0.saturating_add(rhs).into();
  }
}

impl SubAssign<i8> for Parameter {
  fn sub_assign(&mut self, rhs: i8) {
    *self = self.0.saturating_sub(rhs).into();
  }
}

impl From<midly::num::u7> for Parameter {
  fn from(n: midly::num::u7) -> Self {
    (i8::try_from(u8::from(n)).unwrap() + Self::MIN).into()
  }
}

impl FromStr for Parameter {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let value = s.parse()?;

    if value < Self::MIN {
      return Err(NegativeOverflowError { value }.build());
    }

    if value > Self::MAX {
      return Err(PositiveOverflowError { value }.build());
    }

    Ok(Self(value))
  }
}

impl Parameter {
  const MAX: i8 = 63;
  const MIN: i8 = -64;

  pub(crate) fn unipolar(self) -> f32 {
    f32::from(self.0 + 64) / 127.0
  }

  pub(crate) fn value(self) -> i8 {
    self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unipolar() {
    assert_eq!(Parameter::from(Parameter::MIN).unipolar(), 0.0);
    assert_eq!(Parameter::from(Parameter::MAX).unipolar(), 1.0);
  }
}
