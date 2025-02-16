use super::*;

mod run;
mod test;

#[derive(Default, Parser)]
pub(crate) enum Subcommand {
  #[default]
  Run,
  Test,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Subcommand::Run => run::run(options),
      Subcommand::Test => {
        test::run();
        Ok(())
      }
    }
  }
}
