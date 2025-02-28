use super::*;

mod probe;
mod run;
mod test;

#[derive(Default, Parser)]
pub(crate) enum Subcommand {
  Probe,
  #[default]
  Run,
  Test,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Subcommand::Probe => probe::run(),
      Subcommand::Run => run::run(options),
      Subcommand::Test => {
        test::run();
        Ok(())
      }
    }
  }
}
