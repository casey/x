use super::*;

#[derive(Default)]
pub(crate) struct State {
  pub(crate) alpha: Parameter,
  pub(crate) db: f32,
  pub(crate) filter: Filter,
  pub(crate) filters: Vec<Filter>,
  pub(crate) parameter: Parameter,
  pub(crate) text: Option<Text>,
}

impl State {
  pub(crate) fn bottom(mut self) -> Self {
    self.filter.field = Field::Bottom;
    self
  }

  pub(crate) fn circle(mut self) -> Self {
    self.filter.field = Field::Circle;
    self
  }

  pub(crate) fn db(mut self, db: i8) -> Self {
    self.db = db as f32;
    self
  }

  pub(crate) fn frequencies(mut self) -> Self {
    self.filter.field = Field::Frequencies;
    self
  }

  pub(crate) fn invert(mut self) -> Self {
    self.filter.color = invert_color();
    self
  }

  pub(crate) fn invert_r(mut self) -> Self {
    self.filter.color = Mat4f::from_diagonal(&Vec4f::new(-1.0, 1.0, 1.0, 1.0));
    self
  }

  pub(crate) fn push(mut self) -> Self {
    self.filters.push(self.filter.clone());
    self
  }

  pub(crate) fn samples(mut self) -> Self {
    self.filter.field = Field::Samples;
    self
  }

  pub(crate) fn scale(mut self, n: f32) -> Self {
    self.filter.position *= Mat3f::new_scaling(n);
    self
  }

  pub(crate) fn text(mut self, text: Option<Text>) -> Self {
    self.text = text;
    self
  }

  pub(crate) fn times(mut self, n: usize) -> Self {
    for _ in 0..n {
      self = self.push();
    }
    self
  }

  pub(crate) fn top(mut self) -> Self {
    self.filter.field = Field::Top;
    self
  }

  pub(crate) fn x(mut self) -> Self {
    self.filter.field = Field::X;
    self
  }
}
