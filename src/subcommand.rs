use super::*;

mod probe;
mod run;
mod shader;

#[derive(Default, Parser)]
pub(crate) enum Subcommand {
  Probe,
  #[default]
  Run,
  Shader,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Probe => probe::run(),
      Self::Shader => shader::run(),
      Self::Run => run::run(options),
    }
  }
}
